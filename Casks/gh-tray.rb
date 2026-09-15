cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.11"
  sha256 arm:   "a6973f216eed1534753b0167a93d9303e511ba4a15361456f7077abc409ae21b",
         intel: "cd5c23c314f7a1ef464181673e67d6008efd099bc7a9bf8d89ab18316d44e93e"

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
