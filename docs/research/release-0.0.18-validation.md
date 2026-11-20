# Lenz 0.0.18 beta validation — 2026-11-20

Release source: `7e32b7cecaf526968c87fed6cf713b2c2f1a8116`, annotated tag
`v0.0.18`, after rebase-merging PR #56 (one commit, `1064de9`).
[Lenz 0.0.18 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.18)
was published at 09:22:21 UTC on 2026-11-20.
Raw logs are retained in `perf-results/release-0.0.18/` (`perf-check.log`, `push-main.log`,
`old-git*.log`, `publish-watch.log`).

## Local gates

PR #56's head passes `scripts/ci-local.sh` before the merge, and through the pre-push
hook when the branch was pushed. The rebase-merged tree is identical to that head. The
release commit passes the gate again through the pre-push hook on pushing `main`: 57
Cargo test-result groups with zero failures, 87 Python checks with one platform skip,
strict Clippy, panic budgets, standalone TUI release builds and the production size
check (stripped GUI 69,051,168 bytes); the supervisor returns 0 in 338 seconds.
`scripts/perf-check.sh` passes, exit 0. All builds use repository Cargo profiles and no
compiler or profile overrides.

## Older git

The release changes how every guarded git call is built, so the `--old-git` container
step (Debian bullseye, git 2.30.2) ran on a clean `git archive` of the release commit.
`cargo test -p lenz-content` passes all 351 tests. `cargo test -p lenz --no-fail-fast`
passes 697 and fails four `conflicts::tests` cases
(`kinds_by_stage_shape_and_attributes`, `mark_resolved_as_a_delete_and_reopen`,
`merge_file_rebuilds_gits_markers`, `merge_file_without_a_base`). The same four fail on
the 0.0.17 source under the same git, so they are not regressions of this release; they
depend on git 2.35+ (`zdiff3`) and 2.43+ (`merge-file --object-id`) behaviour and
remain open.

## Feature performance and coverage

Measured on release builds before and after the change, same machine, under load:

| Workload | 0.0.17 | 0.0.18 |
| --- | --- | --- |
| zip, 5000 × 2 KiB members | 3.4 ms | 46.9 ms |
| zip, 32 × 2 MiB members | 2.9 ms | 113.2 ms |
| tar.gz, two sizes | 28.4 / 64.5 ms | 39.1 / 119.3 ms |
| tar.zst | 18.8 ms | 29.9 ms |
| PostgreSQL syntax check | ~1 µs | ~30 µs |
| Folder listing | — | +2.3 ms per level |
| SVG load | — | +2.4 ms (~2%) |

The archive cost comes from reading and hashing every member; the PostgreSQL cost from
running the parser on a dedicated large-stack thread. Both trade-offs were accepted
before the change. Startup and interaction benchmarks were not rerun.

## Native packages and publication

Native builds: [44707134343](https://github.com/aantoniadis/lenz/actions/runs/44707134343).
Verification/publisher: [44707161917](https://github.com/aantoniadis/lenz/actions/runs/44707161917),
the 0.0.17 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.18` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `1713105fc294565002566031a2be60264ec73c1a1f82bfc380a9b82109d86638` |
| Linux ARM64 | `383cc5b981db8fd2f94b602eef2f003393ab4788d737b0220ae7764380a4877c` |
| Linux x86_64 | `d0bbb72f9a0b407467f3d05a46b28620955ee7b1ec68b9500158b337bc161f18` |

Homebrew and repository Arch metadata use the published sidecars, which match the
release's asset digests; Ruby and shell syntax checks pass. A fresh Homebrew
installation on every supported platform is not claimed.

## Limits

Native AppKit/macOS interaction, physical displays, trackpads, Wayland interaction,
the fish shell, real GitHub Enterprise hosts, fresh Homebrew installs on each platform
and prolonged live-provider sessions remain unverified. Native CLI/signature checks do
not establish interactive macOS acceptance. macOS packages are ad-hoc signed. AUR
repository metadata is updated separately; an AUR submission is not claimed.
