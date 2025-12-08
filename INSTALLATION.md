# Installation Guide

## macOS

### Download
Download `mercury-macos.dmg` from the [latest release](https://github.com/Harry-kp/mercury/releases).

### First Launch
When you try to open Mercury for the first time, macOS may show a warning: **"Mercury.app" is damaged and can't be opened.**

This happens because the app is not notarized with Apple. The app is safe to use - it's ad-hoc signed and open source.

**To open Mercury:**

#### Option 1: Right-click method (Recommended)
1. Mount the DMG file
2. **Right-click** (or Control+click) on Mercury.app
3. Select **"Open"** from the menu
4. Click **"Open"** in the dialog that appears
5. The app will now open and remember this choice

#### Option 2: System Settings method
1. Try to open the app normally (it will be blocked)
2. Go to **System Settings → Privacy & Security**
3. Scroll down to the Security section
4. Click **"Open Anyway"** next to the Mercury warning
5. Click **"Open"** in the confirmation dialog

#### Option 3: Terminal method (Advanced)
```bash
# Remove the quarantine attribute
xattr -d com.apple.quarantine /Volumes/Mercury/Mercury.app

# Or after copying to Applications:
xattr -d com.apple.quarantine /Applications/Mercury.app
```

### Installation
Drag **Mercury.app** to your Applications folder.

---

## Windows

### Download
Download `mercury.exe` from the [latest release](https://github.com/Harry-kp/mercury/releases).

### First Launch
Windows Defender SmartScreen may show a warning because the app is not signed with a Windows certificate.

**To open Mercury:**
1. Click **"More info"**
2. Click **"Run anyway"**

The app will now run normally.

### Installation
Move `mercury.exe` to your preferred location (e.g., `C:\Program Files\Mercury\`).

Optionally, create a desktop shortcut.

---

## Linux

### Download
Download `mercury-linux.AppImage` from the [latest release](https://github.com/Harry-kp/mercury/releases).

### Installation
```bash
# Make it executable
chmod +x mercury-linux.AppImage

# Run it
./mercury-linux.AppImage

# Optional: Move to /usr/local/bin for system-wide access
sudo mv mercury-linux.AppImage /usr/local/bin/mercury
```

No dependencies needed - AppImage includes everything.

---

## Building from Source

If you prefer to build Mercury yourself:

```bash
# Clone the repository
git clone https://github.com/Harry-kp/mercury.git
cd mercury

# Build release version
cargo build --release

# Run it
./target/release/mercury
```

The binary will be in `target/release/mercury` (or `mercury.exe` on Windows).

---

## Troubleshooting

### macOS: "damaged and can't be opened"
- Use one of the methods above (right-click → Open)
- This is a standard macOS security measure for unsigned apps

### Windows: SmartScreen warning
- Click "More info" → "Run anyway"
- This is normal for unsigned Windows applications

### Linux: Permission denied
- Run: `chmod +x mercury-linux.AppImage`

### Linux: Missing FUSE
If you get a FUSE error, extract and run directly:
```bash
./mercury-linux.AppImage --appimage-extract
./squashfs-root/AppRun
```

---

## Uninstallation

### macOS
1. Drag Mercury.app from Applications to Trash
2. Remove config: `rm -rf ~/.mercury`

### Windows
1. Delete mercury.exe
2. Remove config: Delete `%USERPROFILE%\.mercury`

### Linux
1. Delete the AppImage file
2. Remove config: `rm -rf ~/.mercury`

---

## Security Note

Mercury is **open source** and built from verified code. The "unsigned" warnings are because:
- Apple Developer account costs $99/year for notarization
- Windows Code Signing certificates cost $200+/year

The ad-hoc signed macOS build and unsigned Windows/Linux builds are safe to use. You can verify this by:
1. Checking the source code on GitHub
2. Building from source yourself
3. Reviewing the GitHub Actions workflow that builds releases

All release builds are automated via GitHub Actions from this repository's source code.
