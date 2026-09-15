cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.10"
  sha256 arm:   "39a266cba8ead9ec2b4fded8f9ec96802557c1d914b86ac70406a4b917cc7663",
         intel: "89496cbba83e811a8d654ab420514b0328981cee4e7bb9f63d072ff37b121325"

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
