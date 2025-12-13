---
title: Getting Started
sidebar_label: Getting Started
sidebar_position: 1
---

# Getting Started with Mercury

> Mercury is a fast, minimal API client for developers who value simplicity. 5MB binary. 50ms startup. Zero monthly fees.

## What is Mercury?

Mercury is a **native desktop API client** built with Rust. Unlike Electron-based tools that consume hundreds of megabytes of RAM, Mercury launches instantly and stays light.

Key philosophy:
- **Files, not databases** — Your requests are plain `.http` files you can version control
- **No accounts required** — Your data stays on your machine
- **Keyboard-first** — Send requests with `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux)
- **Live sync** — Edit in VS Code, see changes instantly in Mercury

![Mercury main interface - Replace with: Screenshot showing 3-column layout with sidebar, request panel, and response panel](/img/screenshots/placeholder.png)

## Why Mercury over Postman/Insomnia?

| Feature | Postman | Insomnia | Mercury |
|---------|---------|----------|---------|
| App Size | ~500MB | ~400MB | **5MB** |
| Startup Time | 3-5 sec | 2-4 sec | **&lt;50ms** |
| Memory Usage | 300-800MB | 200-500MB | **~30MB** |
| Account Required | Yes | Yes | **No** |
| Price | $14-25/mo | $5-18/mo | **Free forever** |
| Open Source | No | Partially | **100%** |

## Installation

### macOS
**Option 1: One-Line Installer (Recommended)**
Open your terminal and run:
```bash
curl -sL https://raw.githubusercontent.com/Harry-kp/mercury/master/install.sh | bash
```

**Option 2: Manual Download**
Download the latest `.dmg` from [GitHub Releases](https://github.com/Harry-kp/mercury/releases), open it, and drag Mercury to your Applications folder.

**Option 3: Build from Source**
```bash
cargo install mercury
```

### Windows

Download the latest `.exe` from [GitHub Releases](https://github.com/Harry-kp/mercury/releases) and run it.

### Linux

Download the AppImage from [GitHub Releases](https://github.com/Harry-kp/mercury/releases):

```bash
chmod +x Mercury-*.AppImage
./Mercury-*.AppImage
```

Or build from source:
```bash
cargo install mercury
```

## Your First Request in 60 Seconds

1. **Launch Mercury** — The app opens with a clean workspace

2. **Create a request** — Click the `+` button or press `⌘+N`

3. **Enter the URL** — Type `https://httpbin.org/get` in the URL bar

4. **Send it** — Press `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux)

5. **View the response** — The JSON response appears in the right panel with syntax highlighting

![Creating and sending first request - Replace with: Screenshot or GIF showing the complete workflow](/img/screenshots/placeholder.png)

:::tip Quick Start
Want a faster walkthrough? Check out the [Quick Start guide](/docs/quickstart) for a 5-minute tour.
:::

## Interface Overview

Mercury uses a **3-column layout**:

| Panel | Purpose |
|-------|---------|
| **Sidebar** (left) | Browse your collections and folders |
| **Request** (center) | Edit URL, headers, body, and auth |
| **Response** (right) | View response body, headers, and timing |

![Interface overview - Replace with: Annotated screenshot with arrows pointing to each panel](/img/screenshots/placeholder.png)

## Next Steps

- [Quick Start](/docs/quickstart) — 5-minute guided tour
- [Working with Requests](/docs/features/requests) — Deep dive into the `.http` format
- [Environment Variables](/docs/features/environments) — Use `{{variables}}` in your requests
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts) — Master the keyboard-first workflow
