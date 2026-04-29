cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.8"
  sha256 arm:   "1706889a83177dc8ef4baf2fc687b9070545bd0d4b207a985476410ea601557c",
         intel: "649ac7f7e6dfc75d893869a2c984d795d1d33381416babdad3660e11d68736c1"

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
