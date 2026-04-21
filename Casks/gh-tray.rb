cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.4"
  sha256 arm:   "41e3901183dbc44170f6d51c61e14123aa6ed9715d5507161ff2e897e710722c",
         intel: "0afbbcb3e4f1d3568d2d4d47efeaca9291b6bb3e0ec4f6a6289b1cb99cd9a9a9"

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
