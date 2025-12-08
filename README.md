<p align="center">
  <img src="website/assets/icon.png" alt="Mercury" width="80" height="80">
</p>

<h1 align="center">Mercury</h1>

<p align="center">
  <strong>The last API client you'll ever need.</strong><br>
  5MB. 50ms startup. $0 forever.
</p>

<p align="center">
  <a href="https://github.com/Harry-kp/mercury/releases">Download</a> •
  <a href="#installation">Install</a> •
  <a href="#philosophy">Philosophy</a> •
  <a href="#shortcuts">Shortcuts</a>
</p>

<p align="center">
  <img src="website/assets/screenshot.png" alt="Mercury Screenshot" width="100%" style="border-radius: 8px; border: 1px solid #333;">
</p>

<p align="center">
  <img src="https://img.shields.io/github/v/release/Harry-kp/mercury?style=flat-square&color=00ff88" alt="Release">
  <img src="https://img.shields.io/github/license/Harry-kp/mercury?style=flat-square" alt="License">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue?style=flat-square" alt="Platform">
</p>

---

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

```yaml
# ~/api/users/get-user.http

method: GET
url: https://api.example.com/users/{{user_id}}

headers:
  Authorization: Bearer {{token}}
  Accept: application/json
```

Variables are loaded from `.env` files in your workspace.

---

## Features

- **Collections** — Organize requests in folders
- **Environments** — `.env` file support with `{{variable}}` syntax
- **History** — Timeline of all requests with restore
- **Focus Mode** — Distraction-free editing
- **cURL Import** — Paste cURL commands directly
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
