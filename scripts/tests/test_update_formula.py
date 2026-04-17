"""Test for update_formula.py. Run: python3 scripts/tests/test_update_formula.py"""
from pathlib import Path
import subprocess
import sys
import tempfile

INPUT_FORMULA = '''class GhTray < Formula
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
'''

EXPECTED_FORMULA = '''class GhTray < Formula
  desc "Cross-platform system tray app monitoring GitHub PRs"
  homepage "https://github.com/jsfr/gh-tray-rs"
  version "1.2.3"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3/gh-tray-aarch64-apple-darwin.tar.gz"
      sha256 "aaaa111111111111111111111111111111111111111111111111111111111111"
    end
    on_intel do
      url "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3/gh-tray-x86_64-apple-darwin.tar.gz"
      sha256 "bbbb222222222222222222222222222222222222222222222222222222222222"
    end
  end

  def install
    bin.install "gh-tray"
  end

  test do
    system bin/"gh-tray", "--help"
  end
end
'''

def main():
    repo = Path(__file__).resolve().parents[2]
    script = repo / "scripts" / "update_formula.py"
    with tempfile.NamedTemporaryFile("w", suffix=".rb", delete=False) as f:
        f.write(INPUT_FORMULA)
        formula_path = f.name
    try:
        subprocess.run(
            [sys.executable, str(script),
             "1.2.3",
             "aaaa111111111111111111111111111111111111111111111111111111111111",
             "bbbb222222222222222222222222222222222222222222222222222222222222",
             "https://github.com/jsfr/gh-tray-rs/releases/download/v1.2.3",
             formula_path],
            check=True,
        )
        got = Path(formula_path).read_text()
        if got != EXPECTED_FORMULA:
            print("MISMATCH", file=sys.stderr)
            print("--- expected ---"); print(EXPECTED_FORMULA)
            print("--- got ---"); print(got)
            sys.exit(1)
        print("OK")
    finally:
        Path(formula_path).unlink(missing_ok=True)

if __name__ == "__main__":
    main()
