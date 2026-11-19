# Lenz 0.0.17 beta validation — 2026-11-19

Release source: `3b00d88bc12af90947c356c28d70d8ecba3d49f2`, annotated tag
`v0.0.17`, after rebase-merging PR #55 (four commits, `f37c1b6`..`d889caa`).
[Lenz 0.0.17 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.17)
was published at 12:05:49 UTC on 2026-11-19.
Raw logs are retained in `perf-results/release-0.0.17/` (`ci-prepare.log`, `perf-check.log`).

## Local gates

PR #55's head `a37f0d8` passes `scripts/ci-local.sh` before the merge, and through the
pre-push hook when the branch was pushed. The rebase-merged tree is identical to that
head. The release commit passes the gate again: 57 Cargo test-result groups with zero
failures, 86 Python checks with one platform skip, strict Clippy, panic budgets,
standalone TUI release builds and the production size check (stripped GUI 69,006,368
bytes); the supervisor returns 0 in 339 seconds. The pre-push hook ran it once more on
pushing `main`. `scripts/perf-check.sh` passes, exit 0. All builds use repository Cargo
profiles and no compiler or profile overrides.

## Feature performance and coverage

The release changes how menus, popups, scrollbars and drag selection paint and respond;
it does not change the text, diff, highlighting or search paths that the benchmarks
measure. No before/after benchmark comparison with 0.0.16 was made, and the
interaction and startup benchmarks were not rerun. Completion is the one changed popup
that appears while typing.

New tests cover the menu selection contrast in every built-in theme, the grouped
command menu, the toggle text round trip, palette match ranges, the terminal scrollbar
thumb, the Lenz Code terminal context menu width, drag-selection scrolling in both Lenz
Code adapters, and the explorer and Files list scrollbars. The visual changes were
checked by screenshot on a private X display in Catppuccin Mocha, Light and Tokyo
Night, at a narrow window and a 60-column terminal.

## Native packages and publication

Native builds: [44517034172](https://github.com/aantoniadis/lenz/actions/runs/44517034172).
Verification/publisher: [44517077742](https://github.com/aantoniadis/lenz/actions/runs/44517077742),
the 0.0.16 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.17` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `2be8d6064a29ee21265700089b375f60f743a1ed38af87f7839b83eeaa5acd81` |
| Linux ARM64 | `567902ffc23c179a3df8e2b9b742a70d4848c421ddc6e6cdb6c514bd4ff55787` |
| Linux x86_64 | `fee01b9886cd206dced72657d287015876ff27bdadad68d5dc51181b7db5e9fb` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests; Ruby and shell syntax checks pass. A fresh Homebrew
installation on every supported platform is not claimed.

## Limits

Native AppKit/macOS interaction, physical displays, trackpads, Wayland interaction,
fresh Homebrew installs on each platform and prolonged live-provider sessions remain
unverified. Native CLI/signature checks do not establish interactive macOS
acceptance. macOS packages are ad-hoc signed. AUR repository metadata is updated
separately; an AUR submission is not claimed.
