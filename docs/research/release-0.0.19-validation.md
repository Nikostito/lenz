# Lenz 0.0.19 beta validation — 2026-11-20

Release source: `a48b206a679ee3eb1ee3b88a928178a8da71642c`, annotated tag
`v0.0.19`, after rebase-merging PR #57 (one commit, `6bf3780`).
[Lenz 0.0.19 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.19)
was published at 11:29:56 UTC on 2026-11-20.
Raw logs are retained in `perf-results/release-0.0.19/` (`perf-check.log`, `push-branch.log`,
`push-main.log`, `publish-watch.log`).

## Local gates

PR #57's head passes `scripts/ci-local.sh` before the merge, and through the pre-push
hook when the branch was pushed. The rebase-merged tree is identical to that head. The
release commit passes the gate again through the pre-push hook on pushing `main`: 57
Cargo test-result groups with zero failures, 87 Python checks with one platform skip,
strict Clippy, panic budgets, standalone TUI release builds and the production size
check (stripped GUI 69,050,912 bytes); the supervisor returns 0 in 342 seconds.
`scripts/perf-check.sh` passes, exit 0. All builds use repository Cargo profiles and no
compiler or profile overrides.

## Feature performance and coverage

The release adds Lenz Code's folder trust: Restricted mode, the Folder Trust dialog in
the GUI and the terminal app, the Restricted status item, and project lookups bounded
by the opened folder. New tests cover each held-back server and formatter, one dialog
per folder, a decline surviving a restart, stale results after a folder change, Trust
Parent Folder, the bounded lookups, and each adapter's input path (Enter and Escape
stay restricted, input reaches the dialog first, a 40-column terminal).

`scripts/code-tui-bench.py` (100k lines, 30 rounds, 10 s stress), five interleaved
runs per build against 0.0.18's source: first feedback median 5.00 → 5.16 ms, first
content 14.73 → 15.10 ms with overlapping ranges, input first byte 0.72 → 0.74 ms. RSS
is bimodal (about 44 or 52 MiB) in both builds. The benchmark opens a plain-text file,
so it does not time the folder lookups or the held-back server path. The GUI startup
and interaction benchmarks were not rerun.

## Native packages and publication

Native builds: [44725525255](https://github.com/aantoniadis/lenz/actions/runs/44725525255).
Verification/publisher: [44725529231](https://github.com/aantoniadis/lenz/actions/runs/44725529231),
the 0.0.18 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.19` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `8c92328866dcedf1b304d0e928dae32e6b62f01ec0b4c6847ab60fc28ac30d4b` |
| Linux ARM64 | `83278283e3d258814976f1f9f31215927440de027d63f7eec252598cf7ec30cd` |
| Linux x86_64 | `b06d66fbf373d19b7c677a3f005109decf3abeedc49e44da48f599fea4f71379` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests; Ruby and shell syntax checks pass. A fresh Homebrew
installation on every supported platform is not claimed.

## Limits

Native AppKit/macOS interaction (including Option+T in the dialog), physical displays,
trackpads, Wayland interaction, the fish shell, haml-lint after trusting, fresh
Homebrew installs on each platform and prolonged live-provider sessions remain
unverified. Native CLI/signature checks do not establish interactive macOS acceptance.
macOS packages are ad-hoc signed. AUR repository metadata is updated separately; an
AUR submission is not claimed.
