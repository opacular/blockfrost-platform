# Copilot instructions for blockfrost-platform

This is a Rust workspace (monorepo). Crates live under `crates/` — notably
`crates/gateway` (the WebSocket load-balancing gateway) and `crates/platform`.

## Changelog (required for user-facing changes)

The project keeps [Keep a Changelog](https://keepachangelog.com)-style files:

- Root [`CHANGELOG.md`](../CHANGELOG.md) — product-wide, user-facing changes.
- Per-crate `CHANGELOG.md` (e.g. [`crates/gateway/CHANGELOG.md`](../crates/gateway/CHANGELOG.md)) — changes scoped to that crate.

New entries go under the top `## Unreleased` / `## [Unreleased]` heading, inside
the matching `### Added`, `### Changed`, or `### Fixed` subsection.

**When reviewing a pull request, flag a missing changelog entry** if the PR:

- adds a feature (new HTTP endpoint, new config option, new CLI flag), or
- changes user-facing behavior, or
- fixes a user-visible bug,

…but does not add a corresponding entry under `## Unreleased` in the relevant
`CHANGELOG.md`. Say which file and which `###` section the entry belongs in, and
suggest a one-line entry. A new feature in a crate almost always needs an entry
in that crate's `CHANGELOG.md` (and often the root one too).

Documentation-only, test-only, refactor, and CI/tooling changes do not need a
changelog entry.
