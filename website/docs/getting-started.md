---
title: Getting Started
sidebar_label: Getting Started
sidebar_position: 1
---

# Getting Started with Mercury

> Mercury is a **native** API client that actually feels native. Instant startup. 60fps scrolling. Zero input lag. No subscriptions.

## What is Mercury?

Mercury is a **native desktop API client** built with Rust and GPU-accelerated rendering. Unlike Electron-based tools that feel like slow websites, Mercury responds the instant you click.

Key philosophy:
- **Files, not databases** — Your requests are plain `.json` files you can version control
- **No accounts required** — Your data stays on your machine
- **Keyboard-first** — Send requests with `⌘+Enter` (Mac) or `Ctrl+Enter` (Windows/Linux)
- **Live sync** — Edit in VS Code, see changes instantly in Mercury

![Mercury main interface - Replace with: Screenshot showing 3-column layout with sidebar, request panel, and response panel](/img/screenshots/placeholder.png)

## Why Mercury over Postman/Insomnia?

| Feature | Postman | Insomnia | Mercury |
|:--------|:--------|:---------|:--------|
| **Startup Time** | 3-5 sec | 2-4 sec | **\<300ms** |
| **UI Frame Rate** | Sluggish | Variable | **60fps native** |
| **Input Latency** | 50-100ms | 30-50ms | **\<16ms** |
| **Scrolling** | Janky | Okay | **Buttery smooth** |
| **Binary Size** | ~500MB | ~400MB | **6MB** |
| **Account Required** | Yes | Yes | **No** |
| **Price** | $14-25/mo | $5-18/mo | **Free forever** |
| **Open Source** | No | Partially | **100%** |

## Installation

### ⚡ Fastest Way (30 seconds)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.ps1 | iex
```

**Then launch:**
```bash
mercury
```

:::tip Getting "command not found"?
Restart your terminal or run `source ~/.zshrc` (or `~/.bashrc`) to reload your PATH.
:::

### Want it in your Applications folder? {#applications-folder}

The installer puts `mercury` in `~/.cargo/bin`. Here's how to set it up like a traditional app:

<details>
<summary><strong>macOS: Add to Applications + Dock</strong></summary>

```bash
mkdir -p /Applications/Mercury.app/Contents/MacOS && \
cp ~/.cargo/bin/mercury /Applications/Mercury.app/Contents/MacOS/ && \
cat > /Applications/Mercury.app/Contents/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>mercury</string>
    <key>CFBundleName</key>
    <string>Mercury</string>
    <key>CFBundleIdentifier</key>
    <string>com.mercury.app</string>
</dict>
</plist>
EOF
```
Now search "Mercury" in Spotlight (⌘ Space) and drag to Dock!

</details>

<details>
<summary><strong>Windows: Pin to Start Menu</strong></summary>

1. Open File Explorer → `%USERPROFILE%\.cargo\bin\`
2. Right-click `mercury.exe` → **Create shortcut**
3. Right-click the shortcut → **Pin to Start**

</details>

<details>
<summary><strong>Linux: Add to app launcher</strong></summary>

```bash
cat > ~/.local/share/applications/mercury.desktop << 'EOF'
[Desktop Entry]
Name=Mercury
Exec=$HOME/.cargo/bin/mercury
Type=Application
Categories=Development;
EOF
```

</details>

### Troubleshooting

**macOS Gatekeeper error ("developer cannot be verified"):**
1. Run `mercury` (it will fail)
2. Go to **System Settings → Privacy & Security** → Click **"Allow Anyway"**
3. Run `mercury` again

**Windows SmartScreen:**
Click **"More info"** → **"Run anyway"**

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
- [Working with Requests](/docs/features/requests) — Deep dive into the `.json` format
- [Environment Variables](/docs/features/environments) — Use `{{variables}}` in your requests
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts) — Master the keyboard-first workflow
