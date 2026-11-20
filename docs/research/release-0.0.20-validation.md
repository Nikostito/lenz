# Lenz 0.0.20 beta validation — 2026-11-20

Release source: `343621b9fad077f75941bbd79469586a59492369`, annotated tag
`v0.0.20`, after rebase-merging PR #58 (two commits, `006de1a` and `6dfa672`).
[Lenz 0.0.20 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.20)
was published at 13:33:43 UTC on 2026-11-20.
Raw logs are retained in `perf-results/release-0.0.20/` (`perf-check.log`, `push-branch.log`,
`push-main.log`, `publish-watch.log`) and `perf-results/trust-fixes/`.

## Local gates

PR #58's head passes `scripts/ci-local.sh` through the pre-push hook when the branch was
pushed. The rebase-merged tree is identical to that head. The release commit passes the
gate again through the pre-push hook on pushing `main`: 57 Cargo test-result groups with
zero failures, 87 Python checks with one platform skip, strict Clippy, panic budgets,
standalone TUI release builds and the production size check (stripped GUI 69,059,616
bytes); the supervisor returns 0 in 337 seconds. `scripts/perf-check.sh` passes, exit 0.
All builds use repository Cargo profiles and no compiler or profile overrides.

`server_install_uses_the_selected_node_for_npm_and_validates_before_persisting` is
intermittent under the gate's supervisor ("An installation is already running for this
provider"), also on 0.0.19's source; it passed in both gate runs above and remains open.

## Older git

`scripts/ci-local.sh --old-git` passed while PR #58's first commit was prepared: after the
full gate, its container step (Debian bullseye, git 2.30.2) passed `cargo test -p lenz`
(702) and `-p lenz-content` (353). The commit's last test and README edits were rechecked
on git 2.30 with `cargo test -p lenz conflicts::` (28 passed), not the whole step; the trust
commit was not run on git 2.30. The step now copies the source without build output. Git versions
between 2.30 and 2.56, including Ubuntu 22.04's 2.34, were not run.

## Feature performance and coverage

The release refines Lenz Code's folder trust (answers replacing those for folders inside,
programs bounded by the deciding trusted folder, the dialog asking only about the open
folder, Tab then Enter to trust, Trust Parent Folder hidden for shared folders, changes
from another window) and Re-open on older git. New tests cover each of these in the core
and, for the dialog keys, in both adapters. The added work runs in the background
resolve job and a file-metadata check on the 5-second tick, off the file-open and
first-paint paths; startup and interaction benchmarks were not rerun.

## Native packages and publication

Native builds: [44742309295](https://github.com/aantoniadis/lenz/actions/runs/44742309295).
Verification/publisher: [44742314513](https://github.com/aantoniadis/lenz/actions/runs/44742314513),
the 0.0.19 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.20` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `cc85972928769c3c71699abb21cb53badb731a5f7a44e4e1bb567ff42224806a` |
| Linux ARM64 | `f1682a5fe7d9c52838900d22705101f94a4ff14183bd82364753adcc1d7da3b1` |
| Linux x86_64 | `09cbee5149d1a139f9567164edabae5337cf3344e44b8f898bab4ef1d1ed3cab` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests; Ruby and shell syntax checks pass. A fresh Homebrew
installation on every supported platform is not claimed.

## Limits

Native AppKit/macOS interaction (including the dialog's focus highlight and the folder
ownership check), physical displays, trackpads, Wayland interaction, the fish shell,
fresh Homebrew installs on each platform and prolonged live-provider sessions remain
unverified. Native CLI/signature checks do not establish interactive macOS acceptance.
macOS packages are ad-hoc signed. AUR repository metadata is updated separately; an
AUR submission is not claimed.
