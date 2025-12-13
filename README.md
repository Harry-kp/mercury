<p align="center">
  <img src="assets/icon.png" alt="Mercury" width="80" height="80">
</p>

<h1 align="center">Mercury</h1>

<p align="center">
  <strong>API Testing for Purists.</strong><br>
  5MB. 50ms startup. $0 forever.
</p>

<p align="center">
  <a href="https://harry-kp.github.io/mercury/docs/getting-started">Documentation</a> •
  <a href="https://github.com/Harry-kp/mercury/releases">Download</a> •
  <a href="#philosophy">Philosophy</a> •
  <a href="#shortcuts">Shortcuts</a>
</p>

<p align="center">
  <a href="https://github.com/Harry-kp/mercury/releases"><img src="https://img.shields.io/github/v/release/Harry-kp/mercury?style=flat-square&color=00ff88" alt="Release"></a>
  <a href="https://github.com/Harry-kp/mercury/actions"><img src="https://img.shields.io/github/actions/workflow/status/Harry-kp/mercury/ci.yml?branch=master&style=flat-square&label=build" alt="Build Status"></a>
  <a href="https://github.com/Harry-kp/mercury/blob/master/LICENSE"><img src="https://img.shields.io/github/license/Harry-kp/mercury?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platform">
</p>

<p align="center">
  <a href="https://github.com/Harry-kp/mercury/stargazers"><img src="https://img.shields.io/github/stars/Harry-kp/mercury?style=social" alt="GitHub stars"></a>
  <a href="https://github.com/Harry-kp/mercury/issues"><img src="https://img.shields.io/github/issues/Harry-kp/mercury?style=flat-square" alt="Issues"></a>
  <a href="https://github.com/Harry-kp/mercury/pulls"><img src="https://img.shields.io/github/issues-pr/Harry-kp/mercury?style=flat-square" alt="Pull Requests"></a>
  <a href="https://github.com/Harry-kp/mercury/discussions"><img src="https://img.shields.io/github/discussions/Harry-kp/mercury?style=flat-square" alt="Discussions"></a>
</p>

<p align="center">
  <img src="website/static/img/screenshot.png" alt="Mercury Screenshot" width="100%" style="border-radius: 8px; border: 1px solid #333;">
</p>

## Why Mercury?

| | Postman | Insomnia | **Mercury** |
|---|---|---|---|
| **Size** | ~500MB | ~400MB | **5MB** |
| **Startup** | 3-5 sec | 2-4 sec | **<50ms** |
| **Memory** | 300-800MB | 200-500MB | **~30MB** |
| **Price** | $14-25/mo | $5-18/mo | **Free forever** |
| **Account** | Required | Required | **None** |
| **Telemetry** | Yes | Yes | **None** |

---

## Philosophy

> *"Build half a product, not a half-assed product."* — 37signals

Mercury is built on principles, not features:

- **⚡ Native Rust** — Real performance, not wrapped web pages
- **📁 Files, not databases** — Your requests are just files. Grep them. Git them.
- **🔒 Truly local** — We don't have servers. Your secrets stay yours.
- **⌨️ Keyboard-first** — Your hands never leave the keyboard
- **🚫 No bloat** — No AI, no collaboration, no features you'll never use

---

## Installation

### Download

📦 **[Download latest release](https://github.com/Harry-kp/mercury/releases)** for macOS, Windows, or Linux.

| Platform | Command |
|----------|---------|
| **macOS** | Download `.dmg`, right-click → Open |
| **Windows** | Download `.exe`, click "More info" → "Run anyway" |
| **Linux** | `chmod +x mercury.AppImage && ./mercury.AppImage` |

### Build from Source

```bash
git clone https://github.com/Harry-kp/mercury.git
cd mercury
cargo build --release
./target/release/mercury
```

---

## Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘ Enter` | Send request |
| `⌘ S` | Save request |
| `⌘ K` | Search |
| `⌘ N` | New request |
| `⌘ Shift F` | Focus mode |
| `⌘ H` | History |
| `?` | All shortcuts |

---

## File Format

Your requests are plain text. Version control friendly. No lock-in.

```http
# Simple GET request
GET https://api.example.com/users

# POST with headers and body
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer {{token}}

{
  "name": "John Doe",
  "email": "john@example.com"
}

# Using environment variables
GET {{base_url}}/users/{{user_id}}
Authorization: Bearer {{api_key}}
```

Variables like `{{token}}` are loaded from `.env` files in your workspace.

---

## Features

- **Live File Sync** — Edit in VS Code, updates instantly. Two-way sync.
- **Auto-Save** — Changes are persisted immediately. Never lose work.
- **Collections** — Organize requests in folders
- **Environments** — `.env` file support with `{{variable}}` syntax
- **History** — Timeline of all requests with restore
- **Focus Mode** — Distraction-free editing
- **cURL Import** — Paste cURL commands directly
- **Collection Import** — Import from Postman or Insomnia
- **Syntax Highlighting** — JSON responses beautifully formatted
- **Dark Mode** — Easy on the eyes, built for late nights

---

## What Mercury is NOT

We intentionally don't build:

- ❌ Cloud sync
- ❌ Team collaboration  
- ❌ AI assistants
- ❌ Plugins/extensions
- ❌ User accounts
- ❌ Analytics/telemetry

These aren't missing features. They're features we chose not to build.

---

## Contributing

Mercury is open source. PRs welcome.

```bash
# Development
cargo run

# Tests
cargo test

# Release build
cargo build --release
```

---

## License

MIT License — do whatever you want.

---

<p align="center">
  Built with obsessive minimalism.<br>
  <a href="https://github.com/Harry-kp">@Harry-kp</a>
</p>
