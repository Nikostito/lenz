# Lenz 0.0.23 beta validation — 2026-11-23

Release source: `60509e5f1e43de3418dffc92a9edeca56886507b`, annotated tag
`v0.0.23`, after PR #74 (run the `git` install Lenz finds instead of the macOS stub on
`PATH`).
[Lenz 0.0.23 beta](https://github.com/aantoniadis/lenz/releases/tag/v0.0.23)
was published at 12:07:43 UTC on 2026-11-23.
Raw logs are retained in `perf-results/release-0.0.23/` (`ci-local.log`, `perf-check.log`).

## Local gates

The release commit passes `scripts/ci-local.sh`: 57 Cargo test-result groups with zero
failures, 87 Python checks with one platform skip, strict Clippy, panic budgets,
standalone TUI release builds and the production size check (stripped GUI 80,026,672
bytes, under the 85,000,000-byte budget); the supervisor returns 0 in 343 seconds. The
pre-push hook ran the gate again on pushing `main` (exit 0, 122 seconds, reusing
`target/`). `scripts/perf-check.sh` passes, exit 0. All builds use repository Cargo
profiles and no compiler or profile overrides.

## Feature coverage

`git` resolution is covered by unit tests in `crates/content/src/git.rs` (`PATH` first,
then each install folder once; an old or unreadable `git` passed over; every command
starts the resolved program; its folder added to `PATH` once). It runs once per process and adds one version probe per candidate tried; no
benchmark covers it. A Finder launch on macOS with only Homebrew's git installed, the case
the change fixes, was not run.

## Native packages and publication

Native builds: [45167318810](https://github.com/aantoniadis/lenz/actions/runs/45167318810).
Verification/publisher: [45167377730](https://github.com/aantoniadis/lenz/actions/runs/45167377730),
the 0.0.22 publisher with the version and source commit changed. A first publisher run,
45167339154, started from a commit missing the publisher scripts and was cancelled
before it published anything. The publisher waits for the `automation/validated-v0.0.23`
ref at the release source, requires all three native jobs and their package smoke steps,
verifies all six downloads against release digests, checksum sidecars and build
artifacts, checks embedded source IDs, and runs packaged Linux GUI/TUI launch and editing
checks before publishing the beta.

| Archive target | SHA-256 |
| --- | --- |
| Apple Silicon | `f954f47940c6d54bb144c4f78fe6a35e5f137d71623eff2afa2e2a5899044d90` |
| Linux ARM64 | `75cd69a563c5f9f724c6675d3f53db5a1f117b8e9f4d2062305c064384029526` |
| Linux x86_64 | `6578918fb6359f6afdfb9a636915612e90bc2a2ee4a3bdb5ba682abb9bb9a491` |

Homebrew and repository Arch metadata use the published sidecars, which match the
downloaded archives. A fresh Homebrew installation on every supported platform is not
claimed.

## Limits

Native AppKit/macOS interaction, including the Finder launch this release targets,
physical-display timing, trackpads, fresh Homebrew installs on each platform and
prolonged live-provider sessions remain unverified. Native CLI/signature checks do not
establish interactive macOS acceptance. macOS packages are ad-hoc signed. AUR repository
metadata is updated separately; an AUR submission is not claimed.
