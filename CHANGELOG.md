# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- **Testing**: Expanded unit test coverage to 96 passing tests.

## [0.1.0-beta] - 2025-12-10

### Added
- HTTP methods: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT, TRACE
- Plain text `.http` file format
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
- Native performance (~5MB binary, <50ms startup)
- 100% local storage
- Cross-platform: macOS, Linux, Windows
- **Easy Installation**: One-line installer for macOS (`curl | bash`)
- **Ad-hoc Signing**: Improved Apple Silicon support with hardened runtime entitlements
- No telemetry, no accounts
