# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- A body with no `Content-Type` now says so under the editor, with one click to set JSON, Form or Text. The picker alone was in the corner of the tab row, which is not where anyone looks while typing a body.
- **A body type next to the Body tab** — JSON, Form, Text or none — which writes the `Content-Type` header for you. A JSON body used to be sent with no `Content-Type` at all, so a first POST failed against most APIs. Picking **Form** edits `a=1&b=2` as a table and percent-encodes the values, which is what an OAuth token request needs.
- Mercury reopens the request file you had open when you quit, instead of restoring its contents as an "Untitled" unsaved request.
- **A light theme.** Mercury follows your system appearance and `⌘ D` overrides it. The choice is remembered.
- **A command palette (`⌘ K`)** over every request in the workspace and every command Mercury has. Arrow keys move, `⏎` opens, `Esc` closes.
- The response panel has **Body / Headers / Cookies** tabs instead of checkboxes.
- The request editor's `⋯` menu, the workspace menu and the environment picker replace the old Help/Open menus.

### Fixed
- **A Postman collection with a valueless query parameter failed to import at all.** `?archived` — a parameter Postman exports with no `value` field — aborted the whole import with an error that named the URL parser rather than the parameter. Imports now survive every shape a real export contains.
- **Imported requests arrived without their auth or their body.** Collection-, folder- and request-level Bearer, Basic and API-key auth were dropped, so every imported request came back 401; URL-encoded bodies were dropped, so a login POST sent nothing under a `Content-Type` that promised a form. Both are carried over now, and whatever Mercury genuinely cannot express — OAuth 2, `form-data`, GraphQL — is **named in the notification** instead of vanishing.
- **Importing a collection left the previous request on screen.** The editor still held whatever scratch request was open, often from a folder that was no longer even open, and the collection you just imported was nowhere to be seen. Mercury now opens the first request it imported.
- **An HTML or plain-text response was one endless horizontal line.** Only JSON and XML get re-formatted, so everything else kept the server's single line and had to be read by scrolling sideways through the whole document — which is exactly what you get from a proxy's error page. Unformatted bodies now wrap; formatted JSON still doesn't, so its indentation survives.
- **`⌘ S` on an unsaved request asked you to "Create" a "New request".** It now says Save request / Save.
- **"Imported 3 requests, 1 environments."**
- **Every send raised a toast repeating the status already shown at the top of the response.** The toast is now kept for the cases where the status isn't visible — a response that arrived for a different request, or history covering the pane — and a new send clears the old one, so a stale "404 Not Found" can no longer sit over a connection failure.
- **The URL bar was outlined in amber for an undefined variable that wasn't in the URL.** A `{{token}}` in the headers marked the URL bar, sending you to look in the wrong place, and the explanation was hover-only. The tab that holds the variable now carries the warning.
- **Tab moved the keyboard focus but nothing showed where it had gone.** Every button, tab, list row and the method picker is drawn by hand, and egui only marks focus on the widgets it draws itself. They now paint a focus ring, so the app can be driven from the keyboard without guessing.
- **Sidebar rows, the method picker and the status-bar links were invisible to screen readers.** They reached the accessibility tree as anonymous boxes, so VoiceOver announced nothing for the request list — the app's main navigation. Each one now carries its name.
- **Text that was too faint to read, in both themes.** Hints, counters, timestamps and JSON punctuation sat as low as 2.4:1 against their background; the light theme was the worse of the two. Every text colour now clears WCAG AA (4.5:1) on every surface it lands on, and a test keeps it that way.
- **The delete button's label was white on pink in the dark theme** — 2.7:1, the least readable text in the app.
- **The sidebar grew 2px wider every frame, and sprang back after you resized it.** A text field ended up two pixels wider than the space it was given — a frame occupies `content + margin + 2 × stroke width`, and only the margin was being subtracted. Inside a panel that feeds back into the panel width, so it crept outward forever. The response panel did the same whenever history was open.
- **The macOS app was Apple Silicon only, despite being published as "universal".** Intel Macs could not run the Homebrew cask at all. The bundle is now a real universal binary, and the release fails if it ever isn't.
- **macOS called the app "damaged".** The bundle claimed a sealed signature it did not have, so Gatekeeper refused it outright instead of offering the usual unidentified-developer prompt. It is now ad-hoc signed and sealed, so right-click → Open works.
- **Windows opened a console window behind the GUI.**
- **Compressed responses rendered as mojibake.** Any server with gzip on — and every cURL pasted from a browser, which always asks for gzip — produced garbage. Mercury now decodes gzip, brotli and deflate.
- **`localhost:3000/api` failed with "Invalid URL".** A URL without a scheme gets one: `http` for loopback, `https` otherwise.
- **A request or folder named `../x` was created outside the folder you picked.** Names are now a single file name; `/`, `\`, `..` and a leading `.` are rejected.
- A response that arrived after you opened a different request was shown as *that* request's response. It now goes to history only, and says so.
- Deleting the workspace folder on disk left Mercury claiming it was still open, with buttons that could only fail.
- A 4xx or 5xx was announced with a green tick.
- The response body could not be selected in Pretty mode — only copied whole.
- Binary and large responses told you to "Use Save" without showing one; they now offer a **Save response…** button, and the useless copy icon is gone for bodies that are not text.
- Recent grew without limit in the sidebar while the file kept only 50, and 50 entries pushed the collection off the bottom. It is capped, scrolls, and has a clear button.
- Recent and history rows truncated URLs at a fixed character count instead of the width actually available.
- Failing to open a request printed the whole absolute path across three lines.
- Clicking a folder while filtering silently flipped its expanded state.
- The sidebar filter only searched folders you had already expanded, so a request two folders deep could not be found.
- Pressing Enter in the New request / New folder / Rename dialog did nothing, and the name field did not take focus — you had to click it and then click Create.
- History paired a response with whatever was in the editor when it arrived, not with the request that was actually sent.
- Response headers were capped at ~170px, hiding all but the first few. Headers and Cookies now fill the panel.
- A failed request reported the same error twice, once in the panel and once as a notification.
- The notification after a request read "200 200 OK".
- Formatted JSON wrapped long values onto an unindented next line; it now keeps its structure and scrolls.
- The response panel grew wider when it showed an empty state or a long error, shifting the whole layout.
- At the minimum window size the editor was squeezed to about 50px wide, leaving no room for the URL.
- Icon buttons, tabs and badges had no accessible names, so screen readers announced nothing.
- Sub-millisecond responses showed "0 ms".
- Crash when a URL or error message contained non-ASCII characters.
- Query params with UTF-8 (`%C3%A9`) were decoded incorrectly.
- Copy as cURL included disabled (`#`) headers and didn't escape quotes.
- Pasting a cURL command changed escaped quotes inside single-quoted bodies.
- Typing `?` in a text field opened the shortcuts dialog.
- The shortcuts dialog listed shortcuts that don't exist (⌘I, "Clear Console").
- The "Request completed" message never faded.
- Auto-save didn't run until the first manual ⌘S.
- An open request didn't refresh after being edited outside Mercury.
- Custom auth values couldn't contain trailing spaces while typing.
- The Auth tab could show the previous request's credentials.
- CONNECT/TRACE requests came back as GET after a restart.
- `.env` files in subfolders showed up in the picker but never loaded (only root `.env*` files count now).
- Import failures were silent. Insomnia request names with `/` could write outside the target folder. Postman `{{vars}}` in URL paths were percent-encoded.
- Trailing spaces were stripped from header values.
- Renaming a request without typing `.json` made it disappear from the sidebar.
- Clicking a Recent item could drop the open file's last few seconds of edits.
- **Save** was offered for binary responses restored from history, and wrote a placeholder instead of the file.
- Imported environment keys like `api-key` were renamed to `api_key`, so `{{api-key}}` stopped resolving. Imported values containing quotes were corrupted.
- Editing the selected `.env` file had no effect until you re-selected it.

### Changed
- **The interface was rebuilt.** New palettes (light and dark), a real type scale, hand-drawn vector icons in place of emoji, list rows with hover and selection, a prominent Send button, a filter box in the sidebar, floating notifications, and dialogs on a dimmed backdrop.
- Mercury ships Inter and JetBrains Mono (subset, ~200 KB) so text looks the same on every platform, and no longer scales the whole UI by 1.25.
- `⌘ K` opens the command palette; the sidebar filter box is always visible instead.
- The status bar shows the workspace path relative to `~`.
- Side panel widths are derived from the window size; the minimum window is 900x620 and the default 1280x840.
- **Pretty / Raw** is a two-state switch instead of a link that showed the current mode.
- `⌘ S` on an unsaved request saves into the collection you have selected, not always the workspace root.
- Request files write headers in sorted order, for stable git diffs.
- Error messages include the actual cause (file, reason) instead of generic advice.
- Removed the "About Mercury" menu item, which did nothing.
- Postman import is offered next to Insomnia in the empty sidebar.

### Technical
- `ui/theme.rs` holds both palettes behind one `Theme` shape; `ui/icon.rs` draws every icon from a 24x24 geometry table.
- `AppState` gained a `theme` field (defaults to following the system).
- Flattened the codebase into domain modules, with one implementation per concept (~30% less code).
- Added a headless UI smoke test (egui_kittest).
- Added CLAUDE.md and a `/fix-issue` Claude Code skill. CI now runs on every PR.

## [0.2.0] - 2025-12-20

### Added
- **Centralized Error Handling**: Robust error management with `thiserror` and user-friendly "fading toast" notifications.
- **Request Cancellation**: Dedicated button to abort active or stalled requests instantly.
- **Enhanced cURL Support**: Expanded parser with support for `--user`, `--proxy`, `--cookie`, `--head`, and more.
- **Advanced HTTP Methods**: Support for `CONNECT` and `TRACE` methods in the request editor and dropdown.
- **JSON Beautification**: Added a sparkle icon (✨) for one-click pretty-printing of request bodies.
- **Standardized UI**: Unified iconography across the application (Chevrons, Cross, Checkmarks, and Tooltips).
- **Dot Indicator**: Visual cue for unsaved changes with a descriptive tooltip.

### Changed
- **Unified Navigation**: Replaced folder icons with modern chevrons for consistent collection browsing.
- **Deduplicated UI Components**: Consolidated modal and context menu logic for better consistency.
- **Optimized Rendering**: Reduced memory allocations and improved UI snappiness by minimizing object clones.

### Fixed
- **Icon Rendering**: Resolved "square box" display issues for several emojis (EDIT, WARNING) across different systems.
- **Installation UX**: Refined the one-line installer scripts for smoother onboarding on macOS and Linux.

### Technical
- **Modular Architecture**: Complete codebase restructuring for improved maintainability.
- **Automation**: Integrated `cargo fmt` and `cargo audit` into the development workflow.
- **Testing**: Expanded unit test coverage.

## [0.1.0-beta] - 2025-12-10

### Added
- HTTP methods: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT, TRACE
- Plain-text request files
- Environment variables with `.env` files
- Two-way file sync with external editors
- Request history timeline
- cURL import/export
- Keyboard shortcuts for fast workflow
- Focus mode
- Request search
- Syntax highlighting
- Dark theme
- Folder-based organization

### Technical
- Built with Rust and egui
- Native performance, single small binary
- 100% local storage
- Cross-platform: macOS, Linux, Windows
- **Easy Installation**: One-line installer for macOS (`curl | bash`)
- **Ad-hoc Signing**: Improved Apple Silicon support with hardened runtime entitlements
- No telemetry, no accounts
