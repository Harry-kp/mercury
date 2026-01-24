# Homebrew Cask Setup Guide for Mercury

This guide will help you set up Homebrew Cask distribution for Mercury.

## 📋 Prerequisites

- GitHub account
- Access to create repositories under your account
- Mercury built and ready for distribution

## 🚀 Step-by-Step Setup

### 1. Create Your Homebrew Tap Repository

1. Go to GitHub and create a new repository:
   - **Name:** `homebrew-tap` (required naming convention)
   - **Full URL:** `https://github.com/Harry-kp/homebrew-tap`
   - **Description:** "Homebrew tap for Mercury and other tools"
   - **Public:** Yes (required for Homebrew)

2. Clone the repository:
   ```bash
   git clone https://github.com/Harry-kp/homebrew-tap.git
   cd homebrew-tap
   ```

### 2. Add the Cask Formula

1. Create the Casks directory:
   ```bash
   mkdir -p Casks
   ```

2. Copy the Mercury cask formula:
   ```bash
   cp /path/to/mercury/homebrew/mercury.rb Casks/mercury.rb
   ```

3. Commit and push:
   ```bash
   git add Casks/mercury.rb
   git commit -m "Add Mercury cask"
   git push origin main
   ```

### 3. Build the macOS App Bundle

You need to create a `.app` bundle for distribution. Choose one method:

#### Method A: Using cargo-bundle (Recommended)

```bash
# Install cargo-bundle
cargo install cargo-bundle

# Build the .app bundle
cd /path/to/mercury
cargo bundle --release

# The .app will be created at:
# target/release/bundle/osx/Mercury.app

# Package it for distribution
cd target/release/bundle/osx
tar -czf Mercury-0.2.0-universal-apple-darwin.tar.gz Mercury.app
```

#### Method B: Manual App Bundle

See `homebrew/README.md` for detailed manual instructions.

### 4. Create a GitHub Release

1. Go to your Mercury repository: https://github.com/Harry-kp/mercury/releases

2. Click **Draft a new release**

3. Fill in the details:
   - **Tag:** `v0.2.0`
   - **Title:** `Mercury v0.2.0`
   - **Description:** Release notes

4. **Upload the .app.tar.gz file:**
   - Drag and drop `Mercury-0.2.0-universal-apple-darwin.tar.gz`
   - Or click "Attach binaries" and select the file

5. Click **Publish release**

### 5. Calculate SHA256 and Update Cask

1. Download the file you just uploaded to the release:
   ```bash
   curl -L -O https://github.com/Harry-kp/mercury/releases/download/v0.2.0/Mercury-0.2.0-universal-apple-darwin.tar.gz
   ```

2. Calculate the SHA256:
   ```bash
   shasum -a 256 Mercury-0.2.0-universal-apple-darwin.tar.gz
   ```
   
   Example output:
   ```
   abc123def456... Mercury-0.2.0-universal-apple-darwin.tar.gz
   ```

3. Update your `homebrew-tap/Casks/mercury.rb`:
   ```ruby
   cask "mercury" do
     version "0.2.0"
     sha256 "abc123def456..."  # ← Replace with the actual hash
     
     url "https://github.com/Harry-kp/mercury/releases/download/v#{version}/Mercury-#{version}-universal-apple-darwin.tar.gz"
     # ... rest of the cask
   end
   ```

4. Commit and push:
   ```bash
   cd homebrew-tap
   git add Casks/mercury.rb
   git commit -m "Update Mercury to v0.2.0 with SHA256"
   git push origin main
   ```

### 6. Test the Installation

Test that everything works:

```bash
# Add your tap
brew tap harry-kp/tap

# Install Mercury
brew install --cask harry-kp/tap/mercury

# Launch it
open -a Mercury

# Or run from terminal
mercury
```

If it works, you're done! 🎉

### 7. Update Documentation

The following files have already been updated with Homebrew instructions:
- ✅ `README.md` - Added brew install as recommended method
- ✅ `website/docs/getting-started.md` - Added Homebrew section
- ✅ `Cargo.toml` - Enhanced bundle configuration

## 📦 Release Checklist (For Future Updates)

Every time you release a new version:

- [ ] Update version in `Cargo.toml`
- [ ] Build the .app bundle: `cargo bundle --release`
- [ ] Package it: `tar -czf Mercury-x.x.x-universal-apple-darwin.tar.gz Mercury.app`
- [ ] Create GitHub release and upload the .tar.gz
- [ ] Calculate SHA256 of the uploaded file
- [ ] Update `homebrew-tap/Casks/mercury.rb` with new version and SHA256
- [ ] Push changes to homebrew-tap repo
- [ ] Test installation: `brew upgrade --cask mercury`

## 🔧 Optional: Automate with GitHub Actions

You can automate app bundle creation by adding this to `.github/workflows/release.yml`:

```yaml
- name: Install cargo-bundle
  run: cargo install cargo-bundle

- name: Build macOS App Bundle
  run: |
    cargo bundle --release
    cd target/release/bundle/osx
    tar -czf Mercury-${{ github.ref_name }}-universal-apple-darwin.tar.gz Mercury.app

- name: Calculate SHA256
  id: sha
  run: |
    SHA=$(shasum -a 256 target/release/bundle/osx/Mercury-*.tar.gz | cut -d' ' -f1)
    echo "sha256=$SHA" >> $GITHUB_OUTPUT
    echo "SHA256: $SHA"

- name: Upload App Bundle to Release
  uses: actions/upload-release-asset@v1
  with:
    upload_url: ${{ steps.create_release.outputs.upload_url }}
    asset_path: target/release/bundle/osx/Mercury-${{ github.ref_name }}-universal-apple-darwin.tar.gz
    asset_name: Mercury-${{ github.ref_name }}-universal-apple-darwin.tar.gz
    asset_content_type: application/gzip
```

## 📚 Resources

- [Homebrew Cask Documentation](https://docs.brew.sh/Cask-Cookbook)
- [cargo-bundle on GitHub](https://github.com/burtonageo/cargo-bundle)
- [How to Create a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Homebrew Cask Style Guide](https://docs.brew.sh/Cask-Cookbook#stanza-order)

## 🆘 Troubleshooting

### "Mercury is damaged and can't be opened"

This happens because the app isn't code-signed. Users can fix it with:

```bash
xattr -cr /Applications/Mercury.app
```

Or you can add code signing (requires Apple Developer account):

```bash
codesign --force --deep --sign "Your Developer ID" Mercury.app
```

### "Cask definition is invalid"

Test your cask locally:

```bash
brew audit --cask homebrew-tap/Casks/mercury.rb
brew style homebrew-tap/Casks/mercury.rb
```

### Users getting "checksum mismatch"

The SHA256 in the cask doesn't match the uploaded file. Recalculate:

```bash
curl -L -O [release-url]
shasum -a 256 Mercury-*.tar.gz
```

Then update the cask with the correct hash.

## ✅ Current Status

- ✅ Cask formula created (`homebrew/mercury.rb`)
- ✅ README updated with brew installation
- ✅ Website docs updated
- ✅ Cargo.toml bundle configuration enhanced
- ⏳ Need to: Create homebrew-tap repository
- ⏳ Need to: Build and upload .app bundle
- ⏳ Need to: Calculate SHA256 and update cask

