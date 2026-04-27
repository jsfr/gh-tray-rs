cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.7"
  sha256 arm:   "3a1cd52bbea52efe170830a1fc053c4511187e5c60a5a07f1db9b03ba4a422c9",
         intel: "91561c910f50b525f5ceb3124dde4861362d432c96e249b8b9616bebbb6b4a7a"

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
