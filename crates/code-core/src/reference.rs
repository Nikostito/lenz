//! Copy Reference: the qualified name of the declaration at the caret, written as the language's tools write it
//! (`Foo::Bar::Baz#foo_method` in Ruby, `pkg.mod.Class.method` in Python, `com.acme.Box#open` in Java). It reads the
//! tab's declarations (already extracted for the breadcrumbs) and the few header lines they point at; nothing is
//! parsed again and no language server is asked, so a name the caret only uses (a call) is not resolved.

use lenz_content::highlight::{DeclKind, Declaration, Declarations, Language};

use crate::text::TextSnapshot;

/// A declaration header longer than this is not read: its own name stands for it.
const HEADER_BYTES: usize = 4096;
/// How far above a Ruby method `class << self` (or a file's `package` / `namespace` line) is looked for.
const SCAN_LINES: usize = 2000;

/// The qualified name of the innermost declaration on the caret's line (`at`), or `None` where there is none or it
/// is not a code declaration (a Markdown heading, an HTML element, a CSS rule): Copy Reference then copies the path.
/// `relative` is the file's project-relative path, which names Python modules and Rust module paths.
pub(crate) fn qualified_name(doc: &TextSnapshot, declarations: &Declarations, at: usize, relative: Option<&str>) -> Option<String> {
    let line = u32::try_from(doc.line_of(at.min(doc.len_bytes()))).ok()?;
    let chain = declarations.enclosing_line(line);
    let innermost = chain.last()?;
    if matches!(innermost.kind, DeclKind::Heading | DeclKind::Element | DeclKind::Selector) { return None; }
    let language = doc.language()?;
    let callable = matches!(innermost.kind, DeclKind::Method | DeclKind::Function);
    Some(match language {
        Language::Ruby => ruby(doc, &chain),
        Language::Python => {
            let names: Vec<&str> = chain.iter().map(|d| &*d.name).collect();
            prefixed(relative.and_then(python_module), &names.join("."), ".")
        }
        Language::Java | Language::Kotlin => {
            let (containers, last) = split(&chain, callable);
            let mut name = containers.iter().map(|d| &*d.name).collect::<Vec<_>>().join(".");
            if let Some(last) = last {
                // IntelliJ's Java reference marks a member with `#`; Kotlin writes a plain dot.
                let mark = if language == Language::Java && !containers.is_empty() { "#" } else if containers.is_empty() { "" } else { "." };
                name = format!("{name}{mark}{}", last.name);
            }
            prefixed(header_line(doc, &["package "]).map(|p| p.trim_end_matches(';').trim().to_owned()), &name, ".")
        }
        Language::Rust => {
            let names: Vec<String> = chain.iter().map(|d| if d.kind == DeclKind::Impl { impl_type(&d.name) } else { d.name.to_string() }).collect();
            prefixed(relative.and_then(rust_module), &names.join("::"), "::")
        }
        Language::Php => {
            let (containers, last) = split(&chain, callable);
            let mut name = containers.iter().map(|d| &*d.name).collect::<Vec<_>>().join("\\");
            if let Some(last) = last { name = if containers.is_empty() { last.name.to_string() } else { format!("{name}::{}", last.name) }; }
            prefixed(header_line(doc, &["namespace "]).map(|n| n.trim_end_matches([';', '{']).trim().to_owned()), &name, "\\")
        }
        Language::Cpp => chain.iter().map(|d| &*d.name).collect::<Vec<_>>().join("::"),
        _ => chain.iter().map(|d| &*d.name).collect::<Vec<_>>().join("."),
    })
}

/// The containers and, when the innermost declaration is a function or method, that member apart.
fn split<'a>(chain: &'a [&'a Declaration], callable: bool) -> (&'a [&'a Declaration], Option<&'a Declaration>) {
    match chain.split_last() {
        Some((last, containers)) if callable => (containers, Some(*last)),
        _ => (chain, None),
    }
}

fn prefixed(prefix: Option<String>, name: &str, separator: &str) -> String {
    match prefix.filter(|p| !p.is_empty()) {
        Some(prefix) if !name.is_empty() => format!("{prefix}{separator}{name}"),
        Some(prefix) => prefix,
        None => name.to_owned(),
    }
}

/// Ruby: modules and classes as written (`module Foo::Bar` names `Foo::Bar`, a leading `::` starts again from the top),
/// joined with `::`; an instance method after `#`, a singleton one (`def self.x`, or inside `class << self`) after `.`.
fn ruby(doc: &TextSnapshot, chain: &[&Declaration]) -> String {
    let mut path: Vec<String> = Vec::new();
    let mut member = None;
    for (i, d) in chain.iter().enumerate() {
        let written = header(doc, d).and_then(|h| h.split_whitespace().last().map(str::to_owned)).filter(|w| w.ends_with(&*d.name)).unwrap_or_else(|| d.name.to_string());
        if d.kind == DeclKind::Method && i + 1 == chain.len() {
            match written.rsplit_once('.') {
                Some((_, name)) => member = Some(('.', name.to_owned())),
                None => member = Some((if singleton_block(doc, d) { '.' } else { '#' }, written)),
            }
        } else if let Some(absolute) = written.strip_prefix("::") {
            path = vec![absolute.to_owned()];
        } else {
            path.push(written);
        }
    }
    let path = path.join("::");
    match member {
        Some((_, name)) if path.is_empty() => name,
        Some((mark, name)) => format!("{path}{mark}{name}"),
        None => path,
    }
}

/// Whether a Ruby method sits inside `class << self`: the first less-indented line above it opens that block.
fn singleton_block(doc: &TextSnapshot, method: &Declaration) -> bool {
    let first = method.first_line as usize;
    let indent = |line: usize| doc.line_range(line).and_then(|r| doc.copy_range(r.start..r.end.min(r.start + HEADER_BYTES)));
    let Some(own) = indent(first) else { return false };
    let depth = own.len() - own.trim_start().len();
    for line in (first.saturating_sub(SCAN_LINES)..first).rev() {
        let Some(text) = indent(line) else { return false };
        let body = text.trim_start();
        if body.is_empty() || body.starts_with('#') { continue; }
        if text.len() - body.len() < depth { return body.split_whitespace().collect::<Vec<_>>().starts_with(&["class", "<<", "self"]); }
    }
    false
}

/// The declaration's text from its start to the end of its name (`class Foo::Bar`, `def self.make`).
fn header(doc: &TextSnapshot, d: &Declaration) -> Option<String> {
    let (start, end) = (d.range.start as usize, d.name_range.end as usize);
    if end < start || end - start > HEADER_BYTES { return None; }
    doc.copy_range(start..end)
}

/// The rest of the first line near the top of the file that starts with one of `prefixes` (`package `, `namespace `).
fn header_line(doc: &TextSnapshot, prefixes: &[&str]) -> Option<String> {
    (0..doc.line_count().min(SCAN_LINES)).find_map(|line| {
        let range = doc.line_range(line)?;
        let text = doc.copy_range(range.start..range.end.min(range.start + HEADER_BYTES))?;
        prefixes.iter().find_map(|p| text.trim_start().strip_prefix(p).map(str::to_owned))
    })
}

/// A Python module's dotted name from its project-relative path (`pkg/mod.py` → `pkg.mod`; a package's
/// `__init__.py` names the package).
fn python_module(relative: &str) -> Option<String> {
    let stem = relative.strip_suffix(".py").or_else(|| relative.strip_suffix(".pyi"))?;
    let stem = stem.strip_suffix("/__init__").or(if stem == "__init__" { Some("") } else { None }).unwrap_or(stem);
    Some(stem.replace('/', "."))
}

/// A Rust file's module path below its crate's `src/` (`crates/x/src/a/b.rs` → `crate::a::b`; `lib.rs`, `main.rs`
/// and `mod.rs` name their folder).
fn rust_module(relative: &str) -> Option<String> {
    let inside = relative.rsplit_once("src/").map(|(_, rest)| rest).or_else(|| relative.strip_prefix("src/"))?;
    let stem = inside.strip_suffix(".rs")?;
    let mut parts: Vec<&str> = stem.split('/').collect();
    if parts.last().is_some_and(|last| matches!(*last, "lib" | "main" | "mod")) { parts.pop(); }
    Some(std::iter::once("crate").chain(parts).collect::<Vec<_>>().join("::"))
}

/// The type an `impl` block is for: `impl<T> Show for Box<T>` → `Box`, `impl Session` → `Session`.
fn impl_type(header: &str) -> String {
    let rest = header.rsplit_once(" for ").map_or_else(|| header.trim_start_matches("impl"), |(_, ty)| ty).trim_start();
    // Generic parameters right after `impl` belong to the block, not the type.
    let rest = if rest.starts_with('<') { skip_generics(rest).trim_start() } else { rest };
    let rest = rest.trim_start_matches(['&', ' ']).trim_start_matches("mut ").trim_start_matches("dyn ");
    rest.split(['<', ' ', '{']).next().unwrap_or(rest).to_owned()
}

fn skip_generics(text: &str) -> &str {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => { depth = depth.saturating_sub(1); if depth == 0 { return text.get(i + 1..).unwrap_or_default(); } }
            _ => {}
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, sync::Arc};

    fn reference(path: &str, text: &str, needle: &str) -> Option<String> {
        let document = crate::document::Document::decode(PathBuf::from(format!("/p/{path}")), 1, text.as_bytes());
        let doc = TextSnapshot::from_document(crate::text::BufferId(1), &document);
        let declarations = lenz_content::highlight::declarations_while(doc.language().unwrap(), Arc::from(text), || true).unwrap();
        qualified_name(&doc, &declarations, text.find(needle).unwrap(), Some(path))
    }

    #[test]
    fn ruby_names_nest_modules_and_mark_instance_and_singleton_methods() {
        let text = "module Foo::Bar\n  class Baz < Base\n    LIMIT = 3\n\n    def foo_method(a)\n      a\n    end\n\n    def self.make\n    end\n\n    class << self\n      # a note\n      def build; end\n    end\n\n    def after; end\n  end\n\n  class ::Top\n    def run; end\n  end\nend\n\ndef helper; end\n";
        assert_eq!(reference("lib/baz.rb", text, "foo_method").as_deref(), Some("Foo::Bar::Baz#foo_method"));
        assert_eq!(reference("lib/baz.rb", text, "      a\n").as_deref(), Some("Foo::Bar::Baz#foo_method"), "inside the body");
        assert_eq!(reference("lib/baz.rb", text, "    def foo_method").as_deref(), Some("Foo::Bar::Baz#foo_method"), "the line's start");
        assert_eq!(reference("lib/baz.rb", text, "make").as_deref(), Some("Foo::Bar::Baz.make"));
        assert_eq!(reference("lib/baz.rb", text, "build").as_deref(), Some("Foo::Bar::Baz.build"));
        assert_eq!(reference("lib/baz.rb", text, "after").as_deref(), Some("Foo::Bar::Baz#after"), "after `class << self` ends");
        assert_eq!(reference("lib/baz.rb", text, "class Baz").as_deref(), Some("Foo::Bar::Baz"));
        assert_eq!(reference("lib/baz.rb", text, "run").as_deref(), Some("Top#run"));
        assert_eq!(reference("lib/baz.rb", text, "helper").as_deref(), Some("helper"));
        assert_eq!(reference("lib/baz.rb", "# nothing\n", "nothing"), None);
    }

    #[test]
    fn other_languages_use_their_own_separators_and_module_paths() {
        assert_eq!(reference("pkg/shapes.py", "class Circle:\n    def area(self):\n        return 1\n", "return").as_deref(), Some("pkg.shapes.Circle.area"));
        assert_eq!(reference("pkg/__init__.py", "def f():\n    pass\n", "pass").as_deref(), Some("pkg.f"));
        assert_eq!(reference("src/main/java/A.java", "package com.acme;\n\nclass Box {\n  void open() {}\n}\n", "open").as_deref(), Some("com.acme.Box#open"));
        assert_eq!(reference("crates/x/src/shapes/mod.rs", "struct S;\nimpl<T> Show for S {\n    fn show(&self) {}\n}\n", "show(").as_deref(), Some("crate::shapes::S::show"));
        assert_eq!(reference("src/lib.rs", "impl Session {\n    pub fn new() {}\n}\n", "new").as_deref(), Some("crate::Session::new"));
        assert_eq!(reference("web/app.ts", "class Store {\n  load() {}\n}\n", "load").as_deref(), Some("Store.load"));
        assert_eq!(reference("README.md", "# Title\n\ntext\n", "text"), None, "a heading is not a code reference");
    }

    #[test]
    fn module_paths_and_impl_types() {
        assert_eq!(python_module("a/b/c.py").as_deref(), Some("a.b.c"));
        assert_eq!(python_module("__init__.py").as_deref(), Some(""));
        assert_eq!(python_module("a.rb"), None);
        assert_eq!(rust_module("src/main.rs").as_deref(), Some("crate"));
        assert_eq!(rust_module("crates/core/src/a/b.rs").as_deref(), Some("crate::a::b"));
        assert_eq!(rust_module("build.rs"), None);
        assert_eq!(impl_type("impl<T: Ord> fmt::Display for Wrapper<T>"), "Wrapper");
        assert_eq!(impl_type("impl<'a> View<'a>"), "View");
        assert_eq!(impl_type("impl Session"), "Session");
    }
}
