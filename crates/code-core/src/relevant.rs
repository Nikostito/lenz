//! Go to Test: the test of the active file, or the file a test tests, found by name in the project inventory, as
//! RubyMine's and IntelliJ's Navigate › Test do. Naming conventions decide the names (`user.rb` ↔ `user_spec.rb` /
//! `user_test.rb`, `mod.py` ↔ `test_mod.py`, `Box.java` ↔ `BoxTest.java`, `a.ts` ↔ `a.test.ts`, `x.go` ↔
//! `x_test.go`); among files with such a name, the one whose folders match the file's best wins
//! (`app/models/user.rb` → `spec/models/user_spec.rb` before `spec/requests/user_spec.rb`). Nothing is parsed and no
//! language server is asked; one pass over the inventory per use.

/// Folders that say where code or tests live rather than what they are about: left out when folders are compared.
const ROOTS: &[&str] = &["app", "lib", "src", "spec", "specs", "test", "tests", "__tests__", "main", "java", "kotlin", "scala", "unit", "integration"];

/// What the file is: a test (and the subject names it tests) or a subject (and the test names it may have).
fn names(file: &str) -> (bool, Vec<String>) {
    let (stem, ext) = file.rsplit_once('.').map_or((file, ""), |(s, e)| (s, e));
    let with = |stem: &str, ext: &str| if ext.is_empty() { stem.to_owned() } else { format!("{stem}.{ext}") };
    let scripts: &[&str] = if matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs") { &["ts", "tsx", "js", "jsx", "mjs", "cjs"] } else { &[] };
    let exts: Vec<&str> = if scripts.is_empty() { vec![ext] } else { scripts.to_vec() };
    // Tests.
    for suffix in ["_spec", "_test"] {
        if let Some(subject) = stem.strip_suffix(suffix).filter(|s| !s.is_empty()) {
            // Elixir's tests are `.exs` scripts of `.ex` modules.
            let exts: Vec<&str> = if ext == "exs" { vec!["ex", "exs"] } else { exts.clone() };
            return (true, exts.iter().map(|e| with(subject, e)).collect());
        }
    }
    if let Some(subject) = stem.strip_prefix("test_").filter(|s| !s.is_empty() && ext == "py") { return (true, vec![with(subject, ext)]); }
    for suffix in [".test", ".spec"] {
        if let Some(subject) = stem.strip_suffix(suffix).filter(|s| !s.is_empty()) { return (true, exts.iter().map(|e| with(subject, e)).collect()); }
    }
    if matches!(ext, "java" | "kt" | "kts" | "scala" | "php" | "cs" | "swift" | "groovy") {
        for suffix in ["Tests", "Test", "Spec"] {
            if let Some(subject) = stem.strip_suffix(suffix).filter(|s| !s.is_empty()) { return (true, vec![with(subject, ext)]); }
        }
    }
    // A subject: the names its tests may have.
    let tests = match ext {
        "rb" => vec![with(&format!("{stem}_spec"), ext), with(&format!("{stem}_test"), ext)],
        "py" => vec![with(&format!("test_{stem}"), ext), with(&format!("{stem}_test"), ext)],
        "go" | "rs" | "zig" | "c" | "cc" | "cpp" | "h" | "hpp" => vec![with(&format!("{stem}_test"), ext)],
        "ex" | "exs" => vec![with(&format!("{stem}_test"), "exs")],
        "java" | "kt" | "kts" | "scala" | "php" | "cs" | "swift" | "groovy" => ["Test", "Tests", "Spec"].iter().map(|s| with(&format!("{stem}{s}"), ext)).collect(),
        _ if !scripts.is_empty() => [".test", ".spec"].iter().flat_map(|s| exts.iter().map(move |e| with(&format!("{stem}{s}"), e))).collect(),
        _ => vec![with(&format!("{stem}_test"), ext), with(&format!("{stem}_spec"), ext), with(&format!("test_{stem}"), ext)],
    };
    (false, tests)
}

/// How many folders, from the innermost out, two paths share once the code and test roots are left out.
fn shared(a: &str, b: &str) -> usize {
    let folders = |p: &str| -> Vec<String> {
        let mut parts: Vec<&str> = p.split('/').collect();
        parts.pop();
        parts.into_iter().filter(|f| !ROOTS.contains(f)).map(str::to_owned).collect()
    };
    let (a, b) = (folders(a), folders(b));
    a.iter().rev().zip(b.iter().rev()).take_while(|(x, y)| x == y).count()
}

/// How well a candidate matches: shared folders, the same folder, then a shorter path.
type Rank = (usize, bool, std::cmp::Reverse<usize>);

/// The counterpart of project-relative file `rel` among `files` (the inventory, also project-relative): its test, or
/// the file it tests. `Err` says what was looked for.
pub(crate) fn counterpart<'a>(rel: &str, files: impl IntoIterator<Item = &'a str>) -> Result<String, String> {
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let (test, wanted) = names(file);
    // Best: most shared folders, then the same folder, then the shorter path, then by name.
    let parent = |p: &str| p.rsplit_once('/').map_or("", |(d, _)| d).to_owned();
    let mut best: Option<(Rank, &str)> = None;
    for candidate in files {
        let name = candidate.rsplit('/').next().unwrap_or(candidate);
        if candidate == rel || !wanted.iter().any(|w| w == name) { continue; }
        let key = (shared(rel, candidate), parent(rel) == parent(candidate), std::cmp::Reverse(candidate.len()));
        if best.as_ref().is_none_or(|(k, c)| key > *k || (key == *k && candidate < *c)) { best = Some((key, candidate)); }
    }
    best.map(|(_, c)| c.to_owned()).ok_or_else(|| {
        let what = if test { "No file found for this test" } else { "No test found for this file" };
        format!("{what} (looked for {})", wanted.join(", "))
    })
}

#[cfg(test)]
mod tests {
    use super::counterpart;

    const FILES: &[&str] = &[
        "app/models/user.rb", "spec/models/user_spec.rb", "spec/requests/user_spec.rb", "app/services/foo/bar/baz.rb",
        "spec/services/foo/bar/baz_spec.rb", "lib/tools/report.rb", "test/tools/report_test.rb", "pkg/shapes.py",
        "tests/pkg/test_shapes.py", "src/main/java/com/acme/Box.java", "src/test/java/com/acme/BoxTest.java",
        "web/src/store.ts", "web/src/store.test.ts", "web/src/__tests__/view.test.tsx", "web/src/view.tsx",
        "cmd/server.go", "cmd/server_test.go", "lib/alone.rb", "lib/x/user.rb",
    ];

    fn go(rel: &str) -> Result<String, String> { counterpart(rel, FILES.iter().copied()) }

    #[test]
    fn subjects_find_their_tests_and_tests_their_subjects() {
        assert_eq!(go("app/models/user.rb").as_deref(), Ok("spec/models/user_spec.rb"), "the folder that matches wins");
        assert_eq!(go("spec/models/user_spec.rb").as_deref(), Ok("app/models/user.rb"), "and back, before lib/x/user.rb");
        assert_eq!(go("app/services/foo/bar/baz.rb").as_deref(), Ok("spec/services/foo/bar/baz_spec.rb"));
        assert_eq!(go("lib/tools/report.rb").as_deref(), Ok("test/tools/report_test.rb"));
        assert_eq!(go("test/tools/report_test.rb").as_deref(), Ok("lib/tools/report.rb"));
        assert_eq!(go("pkg/shapes.py").as_deref(), Ok("tests/pkg/test_shapes.py"));
        assert_eq!(go("tests/pkg/test_shapes.py").as_deref(), Ok("pkg/shapes.py"));
        assert_eq!(go("src/main/java/com/acme/Box.java").as_deref(), Ok("src/test/java/com/acme/BoxTest.java"));
        assert_eq!(go("src/test/java/com/acme/BoxTest.java").as_deref(), Ok("src/main/java/com/acme/Box.java"));
        assert_eq!(go("web/src/store.ts").as_deref(), Ok("web/src/store.test.ts"));
        assert_eq!(go("web/src/store.test.ts").as_deref(), Ok("web/src/store.ts"));
        assert_eq!(go("web/src/view.tsx").as_deref(), Ok("web/src/__tests__/view.test.tsx"));
        assert_eq!(go("web/src/__tests__/view.test.tsx").as_deref(), Ok("web/src/view.tsx"));
        assert_eq!(go("cmd/server.go").as_deref(), Ok("cmd/server_test.go"));
        assert_eq!(go("cmd/server_test.go").as_deref(), Ok("cmd/server.go"));
    }

    #[test]
    fn a_file_without_a_counterpart_says_what_was_looked_for() {
        assert_eq!(go("lib/alone.rb"), Err("No test found for this file (looked for alone_spec.rb, alone_test.rb)".into()));
        assert_eq!(go("spec/gone_spec.rb"), Err("No file found for this test (looked for gone.rb)".into()));
    }
}
