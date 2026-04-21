"""Test for update_cask.py. Run: python3 scripts/tests/test_update_cask.py"""
from pathlib import Path
import subprocess
import sys
import tempfile

INPUT_CASK = '''cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "0.0.2"
  sha256 arm:   "0000000000000000000000000000000000000000000000000000000000000000",
         intel: "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
end
'''

EXPECTED_CASK = '''cask "gh-tray" do
  arch arm: "aarch64", intel: "x86_64"

  version "1.2.3"
  sha256 arm:   "aaaa111111111111111111111111111111111111111111111111111111111111",
         intel: "bbbb222222222222222222222222222222222222222222222222222222222222"

  url "https://github.com/jsfr/gh-tray-rs/releases/download/v#{version}/gh-tray-#{arch}-apple-darwin.tar.gz"
  name "gh-tray"
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"

  app "gh-tray.app"
end
'''

def main():
    repo = Path(__file__).resolve().parents[2]
    script = repo / "scripts" / "update_cask.py"
    with tempfile.NamedTemporaryFile("w", suffix=".rb", delete=False) as f:
        f.write(INPUT_CASK)
        cask_path = f.name
    try:
        subprocess.run(
            [sys.executable, str(script),
             "1.2.3",
             "aaaa111111111111111111111111111111111111111111111111111111111111",
             "bbbb222222222222222222222222222222222222222222222222222222222222",
             cask_path],
            check=True,
        )
        got = Path(cask_path).read_text()
        if got != EXPECTED_CASK:
            print("MISMATCH", file=sys.stderr)
            print("--- expected ---"); print(EXPECTED_CASK)
            print("--- got ---"); print(got)
            sys.exit(1)
        print("OK")
    finally:
        Path(cask_path).unlink(missing_ok=True)

if __name__ == "__main__":
    main()
