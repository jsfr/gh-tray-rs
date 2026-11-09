cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.19"
  sha256 arm:   "e8d4e4c7c9566db3e5c320e3bd633f03367f35b641b9e91aca2eb6643eb0bb9f",
         intel: "e158d43abfc3958692cbe40f2c7f0548b0f18dfbc6cd0aa46d03086ab0199eff"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Menu bar app monitoring GitHub pull requests"
  homepage "https://github.com/jsfr/gh-tray-rs"

  depends_on :macos

  app "gh-tray.app"
  binary "#{appdir}/gh-tray.app/Contents/MacOS/gh-tray"

  postflight_steps do
    run "/usr/bin/xattr",
        args: ["-cr", "{{appdir}}/gh-tray.app"]
  end

  zap trash: [
    "~/Library/LaunchAgents/gh-tray.plist",
    "~/Library/LaunchAgents/io.github.jsfr.gh-tray.plist",
    "~/Library/Preferences/io.github.jsfr.gh-tray.plist",
  ]

  caveats <<~EOS
    gh-tray is not code-signed. This cask strips the quarantine attribute
    on install so macOS allows the app to launch.

    If the app still won't open (for example after moving it), re-run:
      xattr -cr "#{appdir}/gh-tray.app"
  EOS
end
