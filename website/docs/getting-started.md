---
title: Getting Started
sidebar_label: Getting Started
sidebar_position: 1
---

# Getting Started

Mercury is a small desktop API client written in Rust with [egui](https://github.com/emilk/egui). It runs as a single native binary on macOS, Windows and Linux.

- **Files, not a database.** A workspace is a folder, and each request is a `.json` file in it. You can edit, grep and commit them.
- **No account.** Mercury has no servers. Your requests stay in your folder, and app state lives in `~/.mercury/`.
- **Keyboard-driven.** Every action in the [shortcut list](/docs/reference/keyboard-shortcuts) works without the mouse.

![Mercury](/img/screenshot.png)

## Installation

### macOS (Homebrew)

```bash
brew install --cask harry-kp/tap/mercury
```

Then launch Mercury from **Applications**, or run `mercury` in a terminal.

### Shell installer

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/Harry-kp/mercury/releases/latest/download/mercury-installer.ps1 | iex
```

Then run:
```bash
mercury
```

If you get "command not found", restart your terminal or run `source ~/.zshrc` (or `~/.bashrc`) to reload your `PATH`.

### Adding it to your Applications folder {#applications-folder}

The shell installer puts `mercury` in `~/.cargo/bin`. To launch it like a regular app:

<details>
<summary><strong>macOS: add to Applications and the Dock</strong></summary>

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
Search for "Mercury" in Spotlight and drag it to the Dock.

</details>

<details>
<summary><strong>Windows: pin to the Start menu</strong></summary>

1. Open File Explorer at `%USERPROFILE%\.cargo\bin\`
2. Right-click `mercury.exe` and choose **Create shortcut**
3. Right-click the shortcut and choose **Pin to Start**

</details>

<details>
<summary><strong>Linux: add to the app launcher</strong></summary>

```bash
cat > ~/.local/share/applications/mercury.desktop << EOF
[Desktop Entry]
Name=Mercury
Exec=$HOME/.cargo/bin/mercury
Type=Application
Categories=Development;
EOF
```

</details>

### Troubleshooting installation

**macOS: "developer cannot be verified"**
1. Run `mercury` once (it will be blocked)
2. Open **System Settings → Privacy & Security** and click **Allow Anyway**
3. Run `mercury` again

**Windows SmartScreen:** click **More info**, then **Run anyway**.

## Next steps

- [Quick Start](/docs/quickstart): send, save and organize your first requests
- [Requests](/docs/features/requests): the editor, params, headers, body and responses
- [Environments](/docs/features/environments): `.env` files and `{{variables}}`
- [Keyboard Shortcuts](/docs/reference/keyboard-shortcuts)
