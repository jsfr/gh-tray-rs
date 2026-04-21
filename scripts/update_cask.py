#!/usr/bin/env python3
"""Rewrite version + sha256 arm:/intel: in a Homebrew Cask file, in place.

Usage: update_cask.py <version> <sha_arm> <sha_intel> <cask_path>
"""
import re
import sys
from pathlib import Path


def rewrite(text: str, version: str, sha_arm: str, sha_intel: str) -> str:
    out = []
    after_arm = False
    version_done = False
    for line in text.splitlines(keepends=True):
        if not version_done and re.match(r'^\s*version\s+"[^"]*"\s*$', line):
            out.append(re.sub(r'"[^"]*"', f'"{version}"', line, count=1))
            version_done = True
            continue
        m = re.match(r'^(\s*sha256\s+arm:\s+)"[^"]*"(,\s*)$', line)
        if m:
            out.append(f'{m.group(1)}"{sha_arm}"{m.group(2)}')
            after_arm = True
            continue
        if after_arm:
            m2 = re.match(r'^(\s*intel:\s+)"[^"]*"(\s*)$', line)
            if m2:
                out.append(f'{m2.group(1)}"{sha_intel}"{m2.group(2)}')
                after_arm = False
                continue
            after_arm = False
        out.append(line)
    return "".join(out)


def main(argv: list[str]) -> int:
    if len(argv) != 5:
        print("usage: update_cask.py <version> <sha_arm> <sha_intel> <cask_path>", file=sys.stderr)
        return 2
    _, version, sha_arm, sha_intel, path = argv
    p = Path(path)
    p.write_text(rewrite(p.read_text(), version, sha_arm, sha_intel))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
