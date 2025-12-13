#!/bin/bash

# Mercury Installer
# Installs the latest version of Mercury and fixes macOS Gatekeeper warnings.

set -e

REPO="Harry-kp/mercury"
INSTALL_DIR="/Applications"
APP_NAME="Mercury.app"

echo "🚀 Starting Mercury Installer..."

# 1. Detect Architecture
ARCH=$(uname -m)
if [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64"
    echo "💻 Detected Intel Mac (x86_64)"
elif [ "$ARCH" = "arm64" ]; then
    TARGET="arm64"
    echo "🍎 Detected Apple Silicon Mac (arm64)"
else
    echo "❌ Unsupported architecture: $ARCH"
    exit 1
fi

# 2. Find Latest Release URL
echo "🔍 Finding latest release..."
LATEST_RELEASE_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*mercury-macos-$TARGET.dmg" | grep -v "sha256" | cut -d : -f 2,3 | tr -d \" | xargs)

if [ -z "$LATEST_RELEASE_URL" ]; then
    echo "❌ Could not find a release for $TARGET. Please check the repository manually."
    exit 1
fi

echo "⬇️  Downloading from: $LATEST_RELEASE_URL"
curl -L -o mercury.dmg "$LATEST_RELEASE_URL"

# 3. Mount DMG
echo "💿 Mounting DMG..."
hdiutil attach mercury.dmg -nobrowse -quiet
MOUNT_POINT="/Volumes/Mercury"

# 4. Install Application
echo "📦 Installing Mercury to $INSTALL_DIR..."
if [ -d "$INSTALL_DIR/$APP_NAME" ]; then
    echo "   Removing existing version..."
    rm -rf "$INSTALL_DIR/$APP_NAME"
fi

cp -R "$MOUNT_POINT/$APP_NAME" "$INSTALL_DIR/"

# 5. Clean Up
echo "🧹 Cleaning up..."
hdiutil detach "$MOUNT_POINT" -quiet
rm mercury.dmg

# 6. The Magic Fix (Quarantine Removal)
echo "✨ Applying magic fix (removing Gatekeeper quarantine)..."
xattr -dr com.apple.quarantine "$INSTALL_DIR/$APP_NAME"
codesign --force --deep --sign - "$INSTALL_DIR/$APP_NAME"

echo "✅ Success! Mercury has been installed."
echo "🎉 You can now open Mercury from your Applications folder."
