#!/usr/bin/env python3
"""Rewrite version + per-arch url/sha256 in a Homebrew formula, in place.

Usage: update_formula.py <version> <sha_arm> <sha_intel> <base_url> <formula_path>
"""
import re
import sys
from pathlib import Path


def rewrite(text: str, version: str, sha_arm: str, sha_intel: str, base_url: str) -> str:
    out = []
    block = None  # None | "arm" | "intel"
    version_done = False
    for line in text.splitlines(keepends=True):
        stripped = line.strip()
        if not version_done and re.match(r'^\s*version\s+"[^"]*"\s*$', line):
            out.append(re.sub(r'"[^"]*"', f'"{version}"', line, count=1))
            version_done = True
            continue
        if stripped.startswith("on_arm do"):
            block = "arm"
        elif stripped.startswith("on_intel do"):
            block = "intel"
        elif stripped == "end" and block is not None:
            block = None
        if block and re.match(r'^\s*url\s+"[^"]*"\s*$', line):
            suffix = "aarch64-apple-darwin.tar.gz" if block == "arm" else "x86_64-apple-darwin.tar.gz"
            out.append(re.sub(r'"[^"]*"', f'"{base_url}/gh-tray-{suffix}"', line, count=1))
            continue
        if block and re.match(r'^\s*sha256\s+"[^"]*"\s*$', line):
            sha = sha_arm if block == "arm" else sha_intel
            out.append(re.sub(r'"[^"]*"', f'"{sha}"', line, count=1))
            continue
        out.append(line)
    return "".join(out)


def main(argv: list[str]) -> int:
    if len(argv) != 6:
        print("usage: update_formula.py <version> <sha_arm> <sha_intel> <base_url> <formula_path>", file=sys.stderr)
        return 2
    _, version, sha_arm, sha_intel, base_url, path = argv
    p = Path(path)
    p.write_text(rewrite(p.read_text(), version, sha_arm, sha_intel, base_url))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
