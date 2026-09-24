# Contributing to Mercury

Thanks for helping. Mercury stays small on purpose, so read this first.

## Philosophy

1. **Say no by default.** Every feature is a liability. Ask whether it can be solved outside the app (git, an editor, the shell).
2. **Build half a product, not a half-assed product.** Do less, and do it well.
3. **Files over databases, conventions over settings.** There is no settings screen and there won't be one.

**Welcome:** bug fixes, performance work, clearer errors, accessibility, docs, and deleting code.
**Discuss first:** UI changes, new shortcuts, file-format changes, new importers.
**Out of scope:** cloud sync, accounts, collaboration, plugins, GraphQL/gRPC/WebSocket, mock servers, test runners, telemetry, and anything that needs a backend.

## Setup

```bash
git clone https://github.com/Harry-kp/mercury && cd mercury
MERCURY_HOME=$(mktemp -d) cargo run   # isolated app data
```

On Linux, install the GTK/xcb dev packages listed in `.github/workflows/ci.yml` first.

## Before you open a PR

CI runs exactly these three, on macOS, Linux and Windows:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- Put tests next to the code (`#[cfg(test)] mod tests`). For UI behavior, extend `src/ui/smoke_test.rs`. It drives the real app headlessly with clicks and keys.
- A bug fix comes with a test that failed before the fix.
- Update the docs listed in the "Keep docs in sync" table in [CLAUDE.md](CLAUDE.md), and add a line to `CHANGELOG.md` under `[Unreleased]`.
- One logical change per PR. Put `Fixes #<issue>` in the description.

[CLAUDE.md](CLAUDE.md) has the code map, the single-source-of-truth rules and the known gotchas. It's written for AI agents, and it's the fastest way for humans to get oriented too.

## Using Claude Code

```
/fix-issue 123
```

This reproduces the bug with a failing test, fixes the root cause, runs the checks, syncs the docs, opens the PR and merges it once CI is green. The steps are in `.claude/skills/fix-issue/SKILL.md`.

## Code of Conduct

This project follows the [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By contributing, you agree your work is licensed under the MIT License.
