# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- **Live File Sync**: Two-way synchronization with file system (edit in VS Code, runs in Mercury)
- **Auto-Save**: Changes persist immediately to disk

## [0.1.0-beta] - 2025-12-07

### Added
- HTTP request execution (GET, POST, PUT, DELETE, PATCH)
- Environment variable support (.env files)
- Request collections with folder organization
- Recent requests (temporary/unsaved requests)
- Breadcrumb navigation
- cURL import via paste detection
- Insomnia collection import
- Response prettification with caching
- Keyboard shortcuts (Cmd+N, Cmd+S, Cmd+Enter, etc.)
- Focus mode (Cmd+Shift+F)
- Timeline/History view (Cmd+H)
- Auto-update check
- Request search (Cmd+K)
- Dark theme with warm color palette
- Workspace-optional architecture

### Technical
- Built with Rust + egui for native performance
- Zero telemetry, no subscriptions
- Local-only data storage (~/.mercury/)
- Cross-platform support (macOS, Windows, Linux)
- Optimized release builds (LTO, strip symbols)
