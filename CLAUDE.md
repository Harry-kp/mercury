# CLAUDE.md

Mercury is a native Rust + egui API client. Requests are plain JSON files in a folder the user opens; environments are `.env` files. Product rule: **say no by default** — no accounts, cloud, telemetry, plugins or settings screens. Small, boring, fast. The shortest correct diff wins.

## Commands (CI runs the last three)
- Run: `MERCURY_HOME=$(mktemp -d) cargo run` (isolated app data; plain `cargo run` uses your real `~/.mercury`)
- One test / module: `cargo test kv::` · `cargo test smoke`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- Docs site: `cd website && npm ci && npm run build` (fails on broken links)

## Where code lives (flat, by domain)
- `src/model.rs` types + on-disk formats · `kv.rs` header/param/Authorization text · `vars.rs` `.env` + `{{var}}`
- `http.rs` send + classify + format · `curl.rs` paste/copy cURL · `import.rs` Postman/Insomnia
- `storage.rs` `~/.mercury/*.json` · `workspace.rs` folder scan, file ops, watcher
- `ui/app.rs` state, frame loop, dialogs, `SHORTCUTS` · `ui/{sidebar,editor,response}.rs` panels
- `ui/widgets.rs` shared widgets + highlighters · `ui/theme.rs` colors, sizes, icons, egui style
- Tests sit in `#[cfg(test)] mod tests` beside the code; UI flows in `src/ui/smoke_test.rs`.

## Single sources of truth (grep before writing a helper)
- Parse header/param/auth text only through `kv.rs`; variables only through `vars.rs`.
- `headers_text` is the truth for headers AND auth; the Auth tab only reads and writes the `Authorization` line.
- Timestamps: `storage::now()`. Truncating user text: `widgets::truncate` (never `&s[..n]`: it panics on UTF-8).
- Colors, sizes, icons: `theme.rs`. Clickable text: `widgets::link`. Dialogs: `widgets::{input_modal, confirm_modal}`.
- Background work: `MercuryApp::spawn` returns an `Event`; never block the UI thread on I/O or the network.
- Errors: `Result<T, String>` carrying the real cause; show them with `self.notify(msg, is_error)`.
- Keyboard shortcuts: only the `SHORTCUTS` table in `ui/app.rs`; the help dialog renders it.

## Compatibility contract
`RequestFile`, `AppState`, `HistoryEntry`, `RecentRequest` (and `HttpResponse` inside history) are read back from users' disks. Never rename or remove a field. New fields get `#[serde(default)]`, plus a test that parses the old JSON (see `storage::tests::reads_v0_2_history_format`).

## Gotchas
- `ctx.input(|i| …)` holds a lock: collect what you need, then act outside the closure (see `handle_keys`).
- Bare-key shortcuts must not fire while typing: check `ctx.wants_keyboard_input()`.
- `eframe` and `egui_kittest` versions move together; dependabot ignores their minor bumps.
- `smoke_test` sets `MERCURY_HOME` for the whole process, so keep it the only test that touches storage on disk.
- Disabled `# Header` lines are not saved to request files (the file stores a map). This is a known limitation, not a bug.
- A gitleaks pre-commit hook may reject credential-looking literals in tests (`-u user:pass`). Build them with `format!`.
- Linux builds need GTK/xcb dev packages (see `.github/workflows/ci.yml`).

## Keep docs in sync (same PR)
| You changed | Also update |
|---|---|
| `SHORTCUTS` | README "Shortcuts", `website/docs/reference/keyboard-shortcuts.md` |
| Request file / `.env` format | README "File format", `website/docs/reference/file-format.md` |
| Anything user-visible | `CHANGELOG.md` → `[Unreleased]`, plus the matching `website/docs/features/*.md` |

## Working on an issue
Run `/fix-issue <number>`: failing test → root-cause fix → checks → docs → PR → green CI → squash-merge.
Done means: the new test failed before the fix and passes after; fmt, clippy and test are clean; docs are synced; CI is green.

## Releasing (only when asked)
Bump `version` in `Cargo.toml`, move the CHANGELOG `[Unreleased]` notes under the new version, commit, then `git tag vX.Y.Z && git push origin vX.Y.Z`. cargo-dist (`release.yml`) builds and publishes the installers. The Homebrew cask is updated by hand: copy the SHA256 that the release job prints into `Harry-kp/homebrew-tap/Casks/mercury.rb`.
