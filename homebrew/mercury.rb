cask "mercury" do
  version "0.2.0"
  sha256 :no_check # Update with actual SHA256 after building

  # Update this URL to point to your .app.tar.gz or .dmg release asset
  url "https://github.com/Harry-kp/mercury/releases/download/v#{version}/Mercury-#{version}-aarch64-apple-darwin.tar.xz"
  name "Mercury"
  desc "Fast, minimal API client for developers"
  homepage "https://github.com/Harry-kp/mercury"

  # Supported macOS versions
  depends_on macos: ">= :monterey"

  # If you have a .app bundle
  app "Mercury.app"

  # Alternative: if you only have a binary, create a simple wrapper
  # binary "mercury"
  
  # Post-install message
  postflight do
    puts <<~EOS
      🚀 Mercury installed successfully!
      
      Launch from Applications or run 'mercury' in terminal.
      
      📚 Documentation: https://harry-kp.github.io/mercury/docs/getting-started
    EOS
  end

  zap trash: [
    "~/Library/Application Support/mercury",
    "~/Library/Preferences/com.mercury.app.plist",
    "~/Library/Saved Application State/com.mercury.app.savedState",
  ]
end

