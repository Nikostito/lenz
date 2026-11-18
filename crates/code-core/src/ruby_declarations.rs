//! Ruby declarations found by name, for Go to Definition where Ruby LSP's index has none: methods made by macros
//! it does not read (ActiveSupport's `cattr_accessor`, `mattr_accessor`, `class_attribute`, `delegate`, and
//! `alias_method`, `define_method`), and, when the server found nothing at all, plain `def`s too. RubyMine lists such
//! declarations in the project and its gems as candidates; this is a line scan by name, not a resolution, so every
//! declaration of the name is a candidate.

use std::path::PathBuf;

/// Macros whose symbol arguments declare methods of those names.
const MACROS: &[&str] = &[
    "attr_reader", "attr_writer", "attr_accessor", "cattr_reader", "cattr_writer", "cattr_accessor", "mattr_reader",
    "mattr_writer", "mattr_accessor", "thread_cattr_reader", "thread_cattr_writer", "thread_cattr_accessor",
    "thread_mattr_reader", "thread_mattr_writer", "thread_mattr_accessor", "class_attribute", "delegate",
    "alias_method", "define_method", "define_singleton_method", "def_delegator", "def_delegators", "alias",
];
/// Files a gem's search reads at most, and the longest file read.
const FILES: usize = 20_000;
const FILE_BYTES: u64 = 2 * 1024 * 1024;

/// The method name a call at the caret names: the identifier, with a `?` or `!` that ends it.
pub(crate) fn method_name(line: &str, word: std::ops::Range<usize>) -> Option<String> {
    let name = line.get(word.clone())?;
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') || name.starts_with(|c: char| c.is_ascii_digit()) { return None; }
    let suffix = line.get(word.end..).and_then(|rest| rest.chars().next()).filter(|c| matches!(c, '?' | '!'));
    Some(suffix.map_or_else(|| name.to_owned(), |s| format!("{name}{s}")))
}

/// Where on `line` a declaration of method `name` (or its writer `name=`) is named: its byte range. A macro's
/// symbol or string argument, or with `defs` a `def` (instance or `self.`).
pub(crate) fn declares(line: &str, name: &str, defs: bool) -> Option<std::ops::Range<usize>> {
    let body = line.trim_start();
    let indent = line.len() - body.len();
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    // The name at `at` in `text`, not part of a longer name (a writer's `=` is allowed after it).
    let named = |text: &str, at: usize| {
        let end = at + name.len();
        text.get(at..end) == Some(name) && !text.get(..at).unwrap_or_default().ends_with(ident)
            && text.get(end..).and_then(|r| r.chars().next()).is_none_or(|c| !ident(c) && (name.ends_with(['?', '!']) || !matches!(c, '?' | '!')))
    };
    if let Some(rest) = body.strip_prefix("def ") {
        if !defs { return None; }
        let rest_at = indent + 4 + (rest.len() - rest.trim_start().len());
        let rest = rest.trim_start();
        let (skip, rest) = rest.strip_prefix("self.").map_or((0, rest), |r| (5, r));
        return named(rest, 0).then(|| rest_at + skip..rest_at + skip + name.len());
    }
    let word = body.split(|c: char| !ident(c)).next().unwrap_or_default();
    if !MACROS.contains(&word) { return None; }
    let args = body.get(word.len()..).unwrap_or_default();
    if !args.starts_with([' ', '(', '\t']) { return None; }
    // `alias new old`: the first word names the new method.
    if word == "alias" {
        let arg = args.trim_start();
        let at = indent + word.len() + (args.len() - arg.len()) + usize::from(arg.starts_with(':'));
        return named(line, at).then(|| at..at + name.len());
    }
    let mut from = indent + word.len();
    while let Some(found) = line.get(from..).and_then(|rest| rest.find(name)) {
        let at = from + found;
        let before = line.get(..at).unwrap_or_default();
        let quoted = before.ends_with([':', '"', '\'']) && !before.ends_with("::");
        // `delegate :a, to: :b` declares `a`, not the target `b`.
        let target = before.trim_end_matches(':').trim_end().ends_with("to:");
        if quoted && !target && named(line, at) { return Some(at..at + name.len()); }
        from = at + name.len();
    }
    None
}

/// The Ruby files of the gems at `roots` (their `lib` and `app` folders) and the project's `files`, at most
/// [`FILES`].
pub(crate) fn files(roots: &[PathBuf], project: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = project.into_iter().filter(|p| p.extension().is_some_and(|e| e == "rb")).collect();
    for root in roots {
        let mut dirs: Vec<PathBuf> = ["lib", "app"].iter().map(|d| root.join(d)).collect();
        while let Some(dir) = dirs.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let Ok(kind) = entry.file_type() else { continue };
                let path = entry.path();
                if kind.is_dir() { dirs.push(path); } else if kind.is_file() && path.extension().is_some_and(|e| e == "rb") { out.push(path); }
                if out.len() >= FILES { return out; }
            }
        }
    }
    out
}

/// The files among `files` that declare `name` (see [`declares`]), each read once; stops when `wanted` turns false
/// or after `limit` files.
pub(crate) fn search(name: &str, files: &[PathBuf], defs: bool, limit: usize, wanted: &dyn Fn() -> bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for path in files {
        if found.len() >= limit || !wanted() { break; }
        if std::fs::metadata(path).is_ok_and(|m| m.len() > FILE_BYTES) { continue; }
        let Ok(bytes) = std::fs::read(path) else { continue };
        let text = String::from_utf8_lossy(&bytes);
        if !text.contains(name) { continue; }
        if text.lines().any(|line| declares(line, name, defs).is_some()) { found.push(path.clone()); }
    }
    found
}


/// The methods a macro line declares: its symbol (or, for `define_method`, string) arguments before the first keyword
/// argument (`delegate :a, :b, to: :c` declares `a` and `b`), or `alias`'s first word.
fn macro_names(line: &str) -> Vec<&str> {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let body = line.trim_start();
    let word = body.split(|c: char| !ident(c)).next().unwrap_or_default();
    let args = body.get(word.len()..).unwrap_or_default();
    if !MACROS.contains(&word) || !args.starts_with([' ', '(', '\t']) { return Vec::new(); }
    let name_at = |text: &str| -> usize { let end = text.find(|c: char| !ident(c)).unwrap_or(text.len()); end + usize::from(text.get(end..).is_some_and(|r| r.starts_with(['?', '!']))) };
    if word == "alias" {
        let arg = args.trim_start().trim_start_matches(':');
        let name = arg.get(..name_at(arg)).unwrap_or_default();
        return if name.is_empty() { Vec::new() } else { vec![name] };
    }
    let mut names = Vec::new();
    for part in args.trim_start_matches(['(', ' ', '\t']).split(',') {
        let part = part.trim();
        // A keyword argument ends the names.
        if part.split_once(':').is_some_and(|(key, _)| !key.is_empty() && key.chars().all(ident)) { break; }
        let text = match part.chars().next() {
            Some(':' | '"' | '\'') => part.get(1..).unwrap_or_default(),
            _ => break,
        };
        let name = text.get(..name_at(text)).unwrap_or_default();
        if !name.is_empty() { names.push(name); }
    }
    names
}

/// A server session's [`GemIndex`], built by the first definition that needs it.
pub(crate) type SharedIndex = std::sync::Arc<std::sync::OnceLock<GemIndex>>;

/// The gems' macro declarations by method name, built once per server session (gems do not change while it runs):
/// a definition then reads only the files that declare the name.
#[derive(Default)]
pub(crate) struct GemIndex {
    files: Vec<PathBuf>,
    names: std::collections::HashMap<Box<str>, Vec<u32>>,
}

impl GemIndex {
    /// Read every Ruby file of the gems at `roots` once.
    pub(crate) fn build(roots: &[PathBuf]) -> Self {
        let mut index = GemIndex::default();
        for path in files(roots, []) {
            if std::fs::metadata(&path).is_ok_and(|m| m.len() > FILE_BYTES) { continue; }
            let Ok(bytes) = std::fs::read(&path) else { continue };
            let text = String::from_utf8_lossy(&bytes);
            let file = index.files.len() as u32;
            let mut any = false;
            for line in text.lines() {
                for name in macro_names(line) {
                    let list = index.names.entry(name.into()).or_default();
                    if list.last() != Some(&file) { list.push(file); }
                    any = true;
                }
            }
            if any { index.files.push(path); }
        }
        index.names.shrink_to_fit();
        index
    }

    /// The gem files whose macros declare `name`.
    pub(crate) fn files_declaring(&self, name: &str) -> Vec<PathBuf> {
        self.names.get(name).into_iter().flatten().filter_map(|&i| self.files.get(i as usize).cloned()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macros_and_defs_declare_their_names() {
        let at = |line: &str, defs: bool| declares(line, "perform_deliveries", defs).map(|r| line[r].to_owned());
        assert_eq!(at("      cattr_accessor :perform_deliveries, default: true", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("    attr_accessor :delivery_handler, :perform_deliveries", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  class_attribute :perform_deliveries", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  delegate :perform_deliveries, to: :config", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  delegate :mail, to: :perform_deliveries", false), None, "the delegation's target is not declared");
        assert_eq!(at("  define_method(:perform_deliveries) { true }", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  alias_method :perform_deliveries, :old", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  alias perform_deliveries old", false).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  def perform_deliveries=(value)", true).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  def self.perform_deliveries", true).as_deref(), Some("perform_deliveries"));
        assert_eq!(at("  def perform_deliveries", false), None, "plain defs only when asked");
        assert_eq!(at("    mail.perform_deliveries = false", true), None, "a use is not a declaration");
        assert_eq!(at("  attr_accessor :perform_deliveries_later", false), None, "a longer name");
        assert_eq!(at("  attr_reader :not_perform_deliveries", false), None);
        assert_eq!(at("  # cattr_accessor :perform_deliveries", false), None, "a comment");
        assert_eq!(declares("  def valid?", "valid?", true).map(|r| r.start), Some(6));
        assert_eq!(declares("  def valid", "valid?", true), None);
        assert_eq!(declares("  def valid?", "valid", true), None, "`valid?` is another method");
    }

    #[test]
    fn a_macro_line_names_its_methods() {
        assert_eq!(macro_names("  delegate :a, :b?, to: :c, prefix: true"), ["a", "b?"]);
        assert_eq!(macro_names("  cattr_accessor :perform_deliveries, default: true"), ["perform_deliveries"]);
        assert_eq!(macro_names("  define_method(\"run!\") { }"), ["run!"]);
        assert_eq!(macro_names("  alias_method :new_name, :old"), ["new_name", "old"], "both are listed; `declares` checks which");
        assert_eq!(macro_names("  alias new_name old"), ["new_name"]);
        assert!(macro_names("  attributes :a").is_empty() && macro_names("  # cattr_accessor :a").is_empty());
    }

    #[test]
    fn a_method_name_keeps_its_question_or_bang() {
        assert_eq!(method_name("x.valid?(1)", 2..7).as_deref(), Some("valid?"));
        assert_eq!(method_name("x.save!", 2..6).as_deref(), Some("save!"));
        assert_eq!(method_name("x.perform_deliveries = 1", 2..20).as_deref(), Some("perform_deliveries"));
        assert_eq!(method_name("x + 1", 2..3), None);
    }

    #[test]
    fn the_search_reads_gem_lib_and_app_folders_and_the_project() {
        let d = tempfile::tempdir().unwrap();
        let gem = d.path().join("actionmailer-8");
        for (p, t) in [("lib/action_mailer/delivery_methods.rb", "module DM\n  cattr_accessor :perform_deliveries\nend\n"), ("lib/other.rb", "x.perform_deliveries\n"), ("test/x.rb", "cattr_accessor :perform_deliveries\n")] {
            std::fs::create_dir_all(gem.join(p).parent().unwrap()).unwrap();
            std::fs::write(gem.join(p), t).unwrap();
        }
        let project = d.path().join("app/mailer.rb");
        std::fs::create_dir_all(project.parent().unwrap()).unwrap();
        std::fs::write(&project, "def perform_deliveries; end\n").unwrap();
        let all = files(std::slice::from_ref(&gem), [project.clone()]);
        assert_eq!(all.len(), 3, "lib and the project, not test/");
        let found = search("perform_deliveries", &all, false, 10, &|| true);
        assert_eq!(found, [gem.join("lib/action_mailer/delivery_methods.rb")]);
        let found = search("perform_deliveries", &all, true, 10, &|| true);
        assert_eq!(found.len(), 2, "with plain defs, the project's too");
        assert!(search("perform_deliveries", &all, true, 10, &|| false).is_empty());
        let index = GemIndex::build(std::slice::from_ref(&gem));
        assert_eq!(index.files_declaring("perform_deliveries"), [gem.join("lib/action_mailer/delivery_methods.rb")]);
        assert!(index.files_declaring("other").is_empty());
    }
}
