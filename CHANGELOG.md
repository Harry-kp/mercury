# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
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
- Request files write headers in sorted order, for stable git diffs.
- Error messages include the actual cause (file, reason) instead of generic advice.
- Removed the "About Mercury" menu item, which did nothing.
- Postman import is offered next to Insomnia in the empty sidebar.

### Technical
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
