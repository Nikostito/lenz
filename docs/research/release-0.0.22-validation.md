# Lenz 0.0.22 beta validation — 2026-11-21

Release source: `a362b04b65fa150dea281089f63c9b832d2a47c4`, annotated tag
`v0.0.22`, after rebase-merging PRs #63–#71 (audit follow-ups, test races, zip comparison,
status-bar baseline, dependency and GPUI v1.29.2 update), #72 (status bar, Preview/Table
switch, Copy feedback, runtime picker) and #73 (Maven and Gradle on JDT LS, Status and
Server Details, Go to Definition on `this`).
[Lenz 0.0.22 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.22)
was published at 14:15:07 UTC on 2026-11-21.
Raw logs are retained in `perf-results/release-0.0.22/` (`perf-check.log`, `push-main.log`,
`publish-watch.log`).

## Local gates

Each PR's head passed `scripts/ci-local.sh` through the pre-push hook when its branch was
pushed; #73 was rebased onto main after #72 merged and passed the gate again (after Clippy
and panic-site budget fixes). The release commit passes the gate through the pre-push hook
on pushing `main`, reusing the existing `target/`: 57 Cargo test-result groups with zero
failures, 87 Python checks with one platform skip, strict Clippy, panic budgets,
standalone TUI release builds and the production size check (stripped GUI 80,022,448
bytes, under the 85,000,000-byte budget raised for tree-sitter-sequel 0.3.13); the
supervisor returns 0 in 369 seconds. `scripts/perf-check.sh` passes, exit 0. All builds
use repository Cargo profiles and no compiler or profile overrides.

## Feature performance and coverage

The GPUI v1.29.2 update was measured against v1.26.2 on the same machine (launch bench
schema 2 focuses new windows as a window manager does); Lenz Code warm launch and Lenz
Diff first paint are unchanged, recorded in the optimization notebook. Zip comparison
skips decompressing unchanged members. The status-bar changes add one short-lived thread
per file load and per quiet re-index start; Lenz Code's file switching has no benchmark and
is unmeasured. The JDT LS behaviour Lenz relies on (code 32/16 diagnostics, Maven profiles
through `globalSettings`, build-file re-import, the re-import's end) was observed with a
protocol probe against JDT LS 1.63.0; Lenz's side is covered by fake-server tests, and the
outside-build message, Maven Profiles and Go to Definition on `this` were checked with the
real server on a Maven fixture. Gradle is untested.

Lenz Diff and Lenz Code were exercised on a headless Wayland compositor (resize, tiling,
tabbing, several windows, focus changes, typing, menus) and by the user on a Hyprland
desktop.

## Native packages and publication

Native builds: [44910781154](https://github.com/aantoniadis/lenz/actions/runs/44910781154).
Verification/publisher: [44910855603](https://github.com/aantoniadis/lenz/actions/runs/44910855603),
the 0.0.21 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.22` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `cb27f2337febdb94650de73f2df9cc0022dc7a9c37f9cc83dd6a58c332ca198d` |
| Linux ARM64 | `2a3da9eb054c2b06176d9ae894e345ef115790a938e0fc5aa82cf8ba5ba346d8` |
| Linux x86_64 | `0d693f8c969c07295a6658544dfa827be741b1aee8aa43af91d2d5fb80774ca4` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests. A fresh Homebrew installation on every supported platform is not
claimed.

## Limits

Native AppKit/macOS interaction, physical-display timing, trackpads, Gradle projects,
the fish shell, fresh Homebrew installs on each platform and prolonged live-provider
sessions remain unverified. Native CLI/signature checks do not establish interactive
macOS acceptance. macOS packages are ad-hoc signed. AUR repository metadata is updated
separately; an AUR submission is not claimed.
