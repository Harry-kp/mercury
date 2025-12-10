# Changelog

All notable changes to Mercury will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- **Live File Sync**: Two-way synchronization with file system (edit in VS Code, runs in Mercury)
- **Auto-Save**: Changes persist immediately to disk

## [0.1.0-beta] - 2025-12-10

### Added
- HTTP methods: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
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
- No telemetry, no accounts
