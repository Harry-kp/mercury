# Mercury Homebrew Cask

This directory contains the Homebrew Cask formula for Mercury.

## Setup Instructions

### 1. Create Your Homebrew Tap Repository

Create a new repository on GitHub named `homebrew-tap`:

```bash
# Repository name must be: homebrew-tap (or homebrew-mercury)
# URL will be: https://github.com/Harry-kp/homebrew-tap
```

### 2. Push the Cask Formula

Copy `mercury.rb` to your tap repository:

```bash
# In your homebrew-tap repository
mkdir -p Casks
cp mercury.rb Casks/mercury.rb
git add Casks/mercury.rb
git commit -m "Add Mercury cask"
git push origin main
```

### 3. Update the Cask Before Each Release

Before releasing a new version, update `mercury.rb`:

1. **Update version number:**
   ```ruby
   version "0.2.0"  # Change this
   ```

2. **Update the download URL** to point to your release asset:
   ```ruby
   url "https://github.com/Harry-kp/mercury/releases/download/v#{version}/Mercury-#{version}-aarch64-apple-darwin.tar.xz"
   ```

3. **Calculate and add SHA256** (important for security):
   ```bash
   # After creating your GitHub release with the .tar.xz file:
   shasum -a 256 Mercury-0.2.0-aarch64-apple-darwin.tar.xz
   ```
   
   Then update the cask:
   ```ruby
   sha256 "abc123..."  # Replace :no_check with actual hash
   ```

### 4. Create a Proper macOS App Bundle

Mercury needs to be packaged as a `.app` bundle for Homebrew Cask. You have two options:

#### Option A: Use cargo-bundle (Recommended)

```bash
cargo install cargo-bundle
cargo bundle --release

# This creates: target/release/bundle/osx/Mercury.app
# Package it:
cd target/release/bundle/osx
tar -czf Mercury-0.2.0-universal-apple-darwin.tar.gz Mercury.app
```

#### Option B: Manual App Bundle

```bash
# Create the .app structure
mkdir -p Mercury.app/Contents/MacOS
mkdir -p Mercury.app/Contents/Resources

# Copy the binary
cp target/release/mercury Mercury.app/Contents/MacOS/

# Copy the icon (use .icns)
cp assets/icons/icon.icns Mercury.app/Contents/Resources/

# Create Info.plist
cat > Mercury.app/Contents/Info.plist << 'EOF'
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
    <key>CFBundleVersion</key>
    <string>0.2.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.2.0</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Package it
tar -czf Mercury-0.2.0-universal-apple-darwin.tar.gz Mercury.app
```

### 5. Upload to GitHub Releases

When creating a new release:

1. Build the `.app` bundle (see above)
2. Upload `Mercury-0.2.0-universal-apple-darwin.tar.gz` to the GitHub release
3. Calculate SHA256 of the uploaded file
4. Update the Cask formula with the correct SHA256
5. Push changes to your homebrew-tap repository

### 6. Testing the Cask

Test locally before publishing:

```bash
# Install from your local formula
brew install --cask --build-from-source ./mercury.rb

# Or test from your tap
brew tap harry-kp/tap
brew install --cask harry-kp/tap/mercury

# Verify it works
open -a Mercury
```

### 7. Users Can Install

Once everything is set up, users can install with:

```bash
brew install --cask harry-kp/tap/mercury
```

Or add it to their Brewfile:

```ruby
tap "harry-kp/tap"
cask "mercury"
```

## Maintaining the Cask

### For Each New Release:

1. Update version in `mercury.rb`
2. Build and upload `.app.tar.gz` to GitHub release
3. Calculate SHA256: `shasum -a 256 Mercury-*.tar.gz`
4. Update SHA256 in `mercury.rb`
5. Commit and push to homebrew-tap repo
6. Users can update with: `brew upgrade --cask mercury`

### Automated Release Workflow (Optional)

Consider adding this to `.github/workflows/release.yml`:

```yaml
- name: Create macOS App Bundle
  run: |
    cargo bundle --release
    cd target/release/bundle/osx
    tar -czf Mercury-${{ github.ref_name }}-universal-apple-darwin.tar.gz Mercury.app
    
- name: Calculate SHA256
  run: |
    shasum -a 256 target/release/bundle/osx/Mercury-*.tar.gz
```

## Resources

- [Homebrew Cask Documentation](https://docs.brew.sh/Cask-Cookbook)
- [cargo-bundle Guide](https://github.com/burtonageo/cargo-bundle)
- [Creating a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)

