cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.6"
  sha256 arm:   "f9e53e0796eed185cf0e08ec384db24484583c7567358eaa5f1e3390c336fc38",
         intel: "a5d45c3a955e3b2f00fb2acc3582d66cf5e2f935f7dd70bd5da26f8ebe790584"

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
