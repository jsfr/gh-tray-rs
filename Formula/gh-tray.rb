class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "0.0.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.3/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "57bc8d34a862de1afecc24cb1ccfd59b3c7ff62b002bd6ca659770b5f4fb5e13"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v0.0.3/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "eef11cedbc1a1eb6facf17e99664def9ef53caae407970f24372aba27808dd2f"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
