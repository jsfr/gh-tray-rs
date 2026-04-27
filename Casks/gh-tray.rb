cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.5"
  sha256 arm:   "af9f31ea2d3a016de012bbd40d9c2bed9010e8d6ce3722a5ca6d7d72f12f1822",
         intel: "e02bb6cfc107865ad1fe4af7f6872e5a172620197f99f587295c895776aeb003"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
  binary "#{appdir}/gh-tray.app/Contents/MacOS/gh-tray"

  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-cr", "#{appdir}/gh-tray.app"],
                   sudo: false
  end

  caveats <<~EOS
    gh-tray is not code-signed. This cask strips the quarantine attribute
    on install so macOS allows the app to launch.

    If the app still won't open (for example after moving it), re-run:
      xattr -cr "#{appdir}/gh-tray.app"
  EOS

  zap trash: [
    "~/Library/LaunchAgents/io.github.jsfr.gh-tray.plist",
    "~/Library/Preferences/io.github.jsfr.gh-tray.plist",
  ]
end
