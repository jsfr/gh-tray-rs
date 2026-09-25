cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.14"
  sha256 arm:   "e506048e1c07ea3884d237c8bd0e5d80e20a84677fb810abd383efb74d05331f",
         intel: "7744015063ade48ffb71dc5b7ef695c62c1d0d7291f7fa5bedba936534345b43"

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
