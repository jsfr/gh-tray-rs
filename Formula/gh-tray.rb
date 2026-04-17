class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "0.0.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.2/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
