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
- `ui/app.rs` state, frame loop, dialogs, `Action`/`SHORTCUTS` · `ui/{sidebar,editor,response}.rs` panels
- `ui/palette.rs` the ⌘K command palette · `ui/widgets.rs` shared widgets + highlighters
- `ui/theme.rs` the two palettes, scales, fonts, egui style · `ui/icon.rs` vector icon geometry
- Tests sit in `#[cfg(test)] mod tests` beside the code; UI flows in `src/ui/smoke_test.rs`.

## Single sources of truth (grep before writing a helper)
- Parse header/param/auth text only through `kv.rs`; variables only through `vars.rs`.
- `headers_text` is the truth for headers, auth AND body type: the Auth tab is a view over the `Authorization` line, the body type picker a view over `Content-Type`. Both go through `kv::{header_value, set_header}`.
- Timestamps: `storage::now()`. Truncating user text: `widgets::truncate` (never `&s[..n]`: it panics on UTF-8).
- Colors, spacing, radii, type sizes: `theme.rs`. Read a color with `theme()`, never a literal — both palettes must stay in sync.
- Icons: `icon.rs`. No emoji anywhere: they render differently on every OS and can't be recolored.
- Buttons, rows, inputs, badges, modals: `widgets.rs` (`row`, `icon_button`, `primary_button`, `text_input`, `tab_button`, `menu_item`, `key_combo`, `input_modal`, `confirm_modal`).
- Commands: the `Action` enum in `ui/app.rs`, run through `MercuryApp::run`. A new command added there shows up in the palette; add it to `SHORTCUTS` too only if it deserves a key.
- Background work: `MercuryApp::spawn` returns an `Event`; never block the UI thread on I/O or the network.
- Errors: `Result<T, String>` carrying the real cause; show them with `self.notify(msg, is_error)`.
- Keyboard shortcuts: only the `SHORTCUTS` table in `ui/app.rs`; the help dialog and the palette render it.

## Compatibility contract
`RequestFile`, `AppState` (including `theme` and `current_file`), `HistoryEntry`, `RecentRequest` (and `HttpResponse` inside history) are read back from users' disks. Never rename or remove a field. New fields get `#[serde(default)]`, plus a test that parses the old JSON (see `storage::tests::reads_v0_2_history_format` and `model::tests::app_state_ignores_legacy_fields`).

## Gotchas
- `ctx.input(|i| …)` holds a lock: collect what you need, then act outside the closure (see `handle_keys`).
- Fonts land one frame late: `ctx.set_fonts` only applies at the next `begin_pass`, and laying out semibold text before then panics inside epaint. `theme::ensure_installed` handles this; don't draw Mercury UI on a context that hasn't been through it.
- A clickable row registers its `ui.interact` rect *after* its contents, or the labels inside swallow the click (its background is reserved up front with `painter.add(Shape::Noop)`). The flip side: a button *inside* a row is unreachable — draw it after the row with `widgets::row_action`. Both directions are covered by `widgets::tests::a_row_action_gets_its_own_clicks`.
- Resizable panels draw their own edge separator from `visuals.widgets.noninteractive.bg_stroke`; don't paint a second border next to it.
- A `SidePanel` grows to fit content (up to its `max_width`), so any text in one must wrap (`Label::wrap`, `horizontal_wrapped`) — a long error or an un-wrapped hint silently widens the panel and shifts the whole layout.
- Inside a right-to-left row, a nested `ui.horizontal` reverses its children and a nested `with_layout(left_to_right)` claims all remaining width. Multi-part widgets (`key_combo`, `switch`) allocate their exact size and paint themselves instead.
- `Response::lost_focus` is how a single-line `TextEdit` reports Enter. Anything that re-grabs focus in the same frame (an "auto-focus if nothing is focused" fallback) erases it — check `lost_focus` first.
- Panel widths come from `Layout::{sidebar,response}_{default,max}` and the window width; fixed widths left the editor unusable at the minimum window size.
- `egui_kittest` pointer simulation only lines up at 1 device pixel per point; `with_pixels_per_point(2.0)` makes clicks miss.
- Names the user types are single file names, never paths: `workspace::safe_name` rejects `/`, `..` and a leading dot. Go through `create_request`/`create_folder`/`rename`, never `parent.join(user_input)`.
- A response is only shown if `current_file` still matches what it was sent from (`sent_file`); otherwise it goes to history. Anything that changes the open request must keep that pairing honest.
- Bare-key shortcuts must not fire while typing: check `ctx.wants_keyboard_input()`.
- `eframe` and `egui_kittest` versions move together; dependabot ignores their minor bumps.
- `smoke_test` sets `MERCURY_HOME` for the whole process, so keep it the only test that touches storage on disk.
- Disabled `# Header` lines are not saved to request files (the file stores a map). This is a known limitation, not a bug.
- A gitleaks pre-commit hook may reject credential-looking literals in tests (`-u user:pass`). Build them with `format!`.
- Linux builds need GTK/xcb dev packages (see `.github/workflows/ci.yml`).
- CI uses the latest stable Rust, which rejects float literals passed as `impl Into<f32>` (write `Stroke::new(1.0_f32, …)`). If CI fails on lints you can't reproduce, run `rustup update stable`.

## Keep docs in sync (same PR)
| You changed | Also update |
|---|---|
| `SHORTCUTS` | README "Shortcuts", `website/docs/reference/keyboard-shortcuts.md` |
| Request file / `.env` format | README "File format", `website/docs/reference/file-format.md` |
| Anything user-visible | `CHANGELOG.md` → `[Unreleased]`, plus the matching `website/docs/features/*.md` |

## Working on an issue
Run `/fix-issue <number>`: failing test → root-cause fix → checks → docs → PR → green CI → squash-merge.
Done means: the new test failed before the fix and passes after; fmt, clippy and test are clean; docs are synced; CI is green.

## Distribution
- The macOS `.app` must be a real universal binary and ad-hoc signed (`codesign --force --deep --sign -`). Unsealed, Gatekeeper calls it "damaged" rather than merely unsigned, and the user has no way through. `release.yml` verifies both and fails the release otherwise.
- Mercury is unsigned by Apple/Microsoft on purpose (no paid certificates). Install docs must tell people what their OS will say and how to get past it.
- App data lives in `~/.mercury`. The Homebrew cask's `zap` list must match it.

## Releasing (only when asked)
Bump `version` in `Cargo.toml`, move the CHANGELOG `[Unreleased]` notes under the new version, commit, then `git tag vX.Y.Z && git push origin vX.Y.Z`. cargo-dist (`release.yml`) builds and publishes the installers. The Homebrew cask is updated by hand: copy the SHA256 that the release job prints into `Harry-kp/homebrew-tap/Casks/mercury.rb`.
