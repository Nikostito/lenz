# Lenz 0.0.16 beta validation — 2026-11-18

Release source: `f8851f72a8a5b677b55326972ed09f4c3a1a3c11`, annotated tag
`v0.0.16`, after squash-merging PR #54 as `f49376e`.
[Lenz 0.0.16 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.16)
was published at 19:42:00 UTC on 2026-11-18.
Raw logs are retained in `perf-results/` (`ci-pr54.log`, `perf-check-0.0.16.log`,
`push-0.0.16.log`).

## Local gates

PR #54's head `cb8d22d` passes `scripts/ci-local.sh` before the merge (supervisor
exit 0 in 677 seconds). The squash-merge tree is identical to that head. The release
commit passes it again through the pre-push hook: 57 Cargo test-result groups with
zero failures, 86 Python checks with one platform skip, strict Clippy, panic budgets,
standalone TUI release builds and the production size check (stripped GUI 68,994,720
bytes); the supervisor returns 0 in 317 seconds. The normal-release
`scripts/perf-check.sh` also passes, exit 0. All builds use repository Cargo profiles
and no compiler or profile overrides.

## Feature performance and coverage

The release adds PR #54's Lenz Code changes. Their performance figures are the
contributor's release-build measurements, recorded in `docs/editor-follow-ups.md`
and not rerun here: project scan 76 → 80 ms on 96k files, Enter after a block opener
19 → 110–150 µs, sync-barrier completion about 1 ms slower at the median, and a first
Ruby definition that builds the gem index in about 70 ms on 817 files. The PR's
real-Ruby-LSP tests are ignored by default and were not rerun. The full 133-case
suite was not rerun.

## Native packages and publication

Native builds: [44380157600](https://github.com/aantoniadis/lenz/actions/runs/44380157600).
Verification/publisher: [44380190228](https://github.com/aantoniadis/lenz/actions/runs/44380190228),
the 0.0.15 publisher with the version and source commit changed. It waits for the
`automation/validated-v0.0.16` ref at the release source, requires all three native
jobs and their package smoke steps, verifies all six downloads against release
digests, checksum sidecars and build artifacts, checks embedded source IDs, and runs
packaged Linux GUI/TUI launch and editing checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `8736d092671303381ea883658c12b108fdc1ab52dce72bcabcee73348697ca56` |
| Linux ARM64 | `e7a22f04abe1ab56abc406a6663a84666707421c83fef45406172ca2822e1879` |
| Linux x86_64 | `82928f41ad66118702a227f5916f359b236dbe0d797a7208a696dfb77c0e82aa` |

Homebrew and repository Arch metadata use the published sidecars; Ruby and shell
syntax checks pass. A fresh Homebrew installation on every supported platform is
not claimed.

## Limits

Native AppKit/macOS interaction, physical displays, trackpads, Wayland interaction,
fresh Homebrew installs on each platform and prolonged live-provider sessions remain
unverified. Native CLI/signature checks do not establish interactive macOS
acceptance. macOS packages are ad-hoc signed. AUR repository metadata is updated
separately; an AUR submission is not claimed.
