---
name: fix-issue
description: Fix a Mercury GitHub issue end to end - reproduce with a failing test, fix the root cause, run checks, sync docs, open a PR, wait for green CI, squash-merge. Use when given an issue number or URL to fix.
argument-hint: <issue number or URL>
---

# Fix issue $ARGUMENTS

Follow CLAUDE.md throughout. Work autonomously. Stop and ask only where a step says **stop**.

## 1. Understand
1. `git status` must be clean (otherwise **stop**). Then `git switch master && git pull --ff-only`.
2. `gh issue view $ARGUMENTS --comments`. Restate the bug as: steps → expected → actual.
3. If it asks for a new feature that conflicts with the product rule (accounts, cloud, plugins, settings screens), or it is too vague to reproduce, **stop**: comment on the issue with what's missing, and report back.
4. `git switch -c fix/<number>-<short-slug>` (use `feat/` for features).

## 2. Reproduce before fixing
1. Find the code: use the map in CLAUDE.md, then `grep`. Read the whole flow the bug touches.
2. Grep every caller of the function you plan to change. Fix it once in the shared function, not in each caller.
3. Write a test that fails for the reported reason:
   - logic: the `#[cfg(test)] mod tests` in that module
   - UI or keyboard behavior: add steps to `src/ui/smoke_test.rs` (egui_kittest clicks and keys through the real frame loop)
4. `cargo test <name>` and confirm it fails **for the right reason**.

## 3. Fix
Make the smallest change that fixes the root cause. Reuse existing helpers (CLAUDE.md, "Single sources of truth"). Add no dependencies. Don't change on-disk formats except in the backward-compatible way CLAUDE.md describes.

## 4. Verify
```
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
```
All three must pass. For visible UI changes, also run `MERCURY_HOME=$(mktemp -d) cargo run` and look. If you screenshot, capture only the Mercury window (`screencapture -l <windowid>`), never the full screen.

## 5. Docs
Apply the "Keep docs in sync" table in CLAUDE.md. Add a line under `CHANGELOG.md` → `[Unreleased]` (Fixed / Added / Changed).

## 6. Self-review
Read `git diff master` as a reviewer would:
- Is this the root cause or a symptom?
- Did any sibling caller stay broken?
- Is there dead code, a duplicated helper, or byte-slicing of user text?
- Did all docs get synced?
If a `/code-review` skill is available, run it and fix what it confirms.

## 7. Ship
1. Commit with a conventional message (`fix: …`) whose body says `Fixes #<number>`.
2. `git push -u origin HEAD`
3. `gh pr create --base master --title "<same as commit>" --body` with these sections: **Root cause**, **Fix**, **Tests** (name the new test), and `Fixes #<number>`.
4. `gh pr checks --watch`. If anything fails: `gh run view <run-id> --log-failed`, fix it, push, and watch again. After 3 failed rounds, **stop** and report.
5. Once every check is green: `gh pr merge --squash --delete-branch`. If the user said not to merge, stop at green instead.
6. `git switch master && git pull --ff-only`.

## 8. Report
Give the PR URL, the root cause in one sentence, the test that proves the fix, and anything you noticed but didn't fix.
