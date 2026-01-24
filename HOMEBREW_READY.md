# ✅ Homebrew Cask Setup Complete!

Mercury is now configured for automated Homebrew Cask distribution!

## What's Been Done

### 1. ✅ Release Workflow Updated
**File:** `.github/workflows/release.yml`

Added a new job `build-macos-app-bundle` that:
- Runs on macOS Apple Silicon (macos-14)
- Installs `cargo-bundle`
- Builds `Mercury.app` bundle
- Packages it as `Mercury-vX.X.X-universal-apple-darwin.tar.gz`
- Calculates SHA256 automatically
- Uploads both the `.tar.gz` and `.sha256` to the GitHub release
- Displays the SHA256 in the workflow logs for easy copying

### 2. ✅ Homebrew Tap Updated
**Repo:** https://github.com/Harry-kp/homebrew-tap

- Updated `Casks/mercury.rb` with correct URL format
- Cask is ready to work with automated releases
- URL format: `Mercury-v#{version}-universal-apple-darwin.tar.gz`

### 3. ✅ Documentation Updated
- README.md - Homebrew listed as recommended installation
- website/docs/getting-started.md - Homebrew section added
- Homebrew badge added to README

## Next Release (v0.3.0 or later)

When you create your next release, the workflow will **automatically**:

1. Build the Mercury.app bundle
2. Package it for Homebrew
3. Upload it to the GitHub release
4. Show you the SHA256 in the workflow logs

Then you just need to:

1. **Copy the SHA256** from the workflow logs (look for the "Homebrew SHA256" notice)
2. **Update your tap:**
   ```bash
   cd ~/homebrew-tap  # or wherever your tap repo is
   # Edit Casks/mercury.rb:
   # - Update version = "0.3.0" (or whatever version)
   # - Update sha256 "abc123..." (paste the SHA256 from logs)
   git commit -am "Update Mercury to v0.3.0"
   git push
   ```

3. **Done!** Users can now install with:
   ```bash
   brew upgrade --cask mercury
   ```

## Testing the Next Release

After you push v0.3.0 (or your next version tag):

1. Wait for the workflow to complete
2. Check the release page: https://github.com/Harry-kp/mercury/releases
3. You should see: `Mercury-v0.3.0-universal-apple-darwin.tar.gz`
4. The workflow logs will show: `Homebrew SHA256: <hash>`
5. Update the tap with that hash
6. Test installation:
   ```bash
   brew upgrade --cask mercury
   ```

## Current Status

**v0.2.0:** ⚠️ Does not have the .app bundle (was released before workflow update)

**v0.3.0+:** ✅ Will automatically include .app bundle for Homebrew

## Manual Build (If Needed)

If you need to add the .app bundle to v0.2.0 manually:

```bash
cd /Users/harrykp/Documents/mercury
cargo bundle --release
cd target/release/bundle/osx
tar -czf Mercury-v0.2.0-universal-apple-darwin.tar.gz Mercury.app
shasum -a 256 Mercury-v0.2.0-universal-apple-darwin.tar.gz

# Then upload to release:
gh release upload v0.2.0 Mercury-v0.2.0-universal-apple-darwin.tar.gz --clobber

# Update the tap with the SHA256
```

## Installation Command

Users can now install Mercury with:

```bash
brew install --cask harry-kp/tap/mercury
```

Or if they already have your tap:

```bash
brew tap harry-kp/tap
brew install --cask mercury
```

## Files Modified

1. `.github/workflows/release.yml` - Added macOS app bundle job
2. `homebrew/mercury.rb` - Local template
3. `homebrew/README.md` - Setup instructions
4. `HOMEBREW_SETUP.md` - Detailed guide
5. `README.md` - Installation instructions
6. `website/docs/getting-started.md` - Documentation
7. `Cargo.toml` - Enhanced bundle metadata

## Useful Commands

```bash
# Test your tap locally
brew tap harry-kp/tap

# Install Mercury
brew install --cask mercury

# Check installation
which mercury
mercury --version

# Update Mercury
brew upgrade --cask mercury

# Uninstall
brew uninstall --cask mercury
```

---

**🎉 Everything is ready!** Your next release will automatically support Homebrew Cask installation.

