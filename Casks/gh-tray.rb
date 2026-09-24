cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.13"
  sha256 arm:   "6cf8173fe2dacd61991ae5e722ba6016b3d5c63d5c4022db639b4167eac4954d",
         intel: "8b1bb2f490c3b1be58e9660ca4a150c32668c76a46bed6a05f5fdeff4910836b"

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
