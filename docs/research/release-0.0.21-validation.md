# Lenz 0.0.21 beta validation — 2026-11-20

Release source: `ade895b13a6a1c11de2ab1681d5195114701cbc7`, annotated tag
`v0.0.21`, after rebase-merging PRs #59 (install tests), #60 (Lenz Code branch picker),
#61 (no language-server memory warning) and #62 (Stay Restricted keeps inner trust).
[Lenz 0.0.21 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.21)
was published at 15:45:59 UTC on 2026-11-20.
Raw logs are retained in `perf-results/release-0.0.21/` (`perf-check.log`, `push-main.log`,
`publish-watch.log`).

## Local gates

Each PR's head passed `scripts/ci-local.sh` through the pre-push hook when its branch was
pushed; each rebase-merged tree is identical to that head. The release commit passes the
gate again, from a clean `target/`, through the pre-push hook on pushing `main`: 57 Cargo
test-result groups with zero failures, 87 Python checks with one platform skip, strict
Clippy, panic budgets, standalone TUI release builds and the production size check
(stripped GUI 69,059,104 bytes); the supervisor returns 0 in 633 seconds.
`scripts/perf-check.sh` passes, exit 0. All builds use repository Cargo profiles and no
compiler or profile overrides.

## Feature performance and coverage

The branch picker reads ahead/behind counts after the lists (about 350 ms for 500
branches far behind, against about 6 ms for the lists), so the lists never wait for
them. The trust and memory changes do not touch the text, diff, highlighting or search
paths the benchmarks measure; startup and interaction benchmarks were not rerun.
New tests cover the picker's sections, folders, favorites, Show more and tracking labels
in the core and both adapters, the dialog's Stay Restricted keeping inner trust, and the
absence of a memory notice.

## Native packages and publication

Native builds: [44762995521](https://github.com/aantoniadis/lenz/actions/runs/44762995521).
Verification/publisher: [44762999960](https://github.com/aantoniadis/lenz/actions/runs/44762999960),
the 0.0.20 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.21` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `b3faf91f3514bc8def128fb6c54e7252f00287dff59e6411a9db53af3e491e39` |
| Linux ARM64 | `f2ecaadafbf41a65215c452c6b9551342c22c2d0c5c63a1686dc3e2060251ca3` |
| Linux x86_64 | `bb5523e93fe9adb0361701aec0f646d5b2c4026c0dacb41ba64011ada503ceb2` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests; Ruby and shell syntax checks pass. A fresh Homebrew
installation on every supported platform is not claimed.

## Limits

Native AppKit/macOS interaction, physical displays, trackpads, Wayland interaction,
the fish shell, fresh Homebrew installs on each platform and prolonged live-provider
sessions remain unverified. Native CLI/signature checks do not establish interactive
macOS acceptance. macOS packages are ad-hoc signed. AUR repository metadata is updated
separately; an AUR submission is not claimed.
