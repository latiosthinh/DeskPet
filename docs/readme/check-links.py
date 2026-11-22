#!/usr/bin/env py
"""Check the README and DEVELOPMENT links, then catch pet-grid drift.

Run from the repo root: py docs/readme/check-links.py
Local files and #anchors are checked. External http(s) URLs are only listed.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ["README.md", "docs/DEVELOPMENT.md"]
MARKDOWN = re.compile(r"!?\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+\"[^\"]*\")?\s*\)")
HTML_ATTR = re.compile(r"(?:src|href)\s*=\s*\"([^\"]+)\"")
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*#*\s*$")
SCHEME = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")


def slug(text):
    out = []
    for ch in text.strip().lower():
        if ch.isalnum() or ch in "-_":
            out.append(ch)
        elif ch == " ":
            out.append("-")
    return "".join(out)


def headings(path):
    found = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        m = HEADING.match(line)
        if m:
            found.add(slug(m.group(2)))
    return found


def links(path):
    text = path.read_text(encoding="utf-8")
    return MARKDOWN.findall(text) + HTML_ATTR.findall(text)


def check_doc(rel, external):
    path = ROOT / rel
    errors = []
    seen = set()
    for target in links(path):
        if target in seen:
            continue
        seen.add(target)
        if SCHEME.match(target):
            if target.startswith(("http://", "https://")):
                external.add(target)
            continue
        file_part, _, anchor = target.partition("#")
        dest = path if not file_part else (path.parent / file_part)
        try:
            dest = dest.resolve()
        except OSError:
            dest = path.parent / file_part
        dest = dest.resolve()
        if not dest.exists():
            errors.append(f"missing file: {target}")
            continue
        if anchor and not slug(anchor) in headings(dest):
            errors.append(f"missing anchor: {target}")
    return errors, path


def grid_errors():
    readme = (ROOT / "README.md").read_text(encoding="utf-8")
    pets = json.loads((ROOT / "website" / "pets.json").read_text(encoding="utf-8"))
    errors = []
    for pet in pets:
        if pet["name"] not in readme:
            errors.append(f"grid: name missing for {pet['id']}: {pet['name']}")
        if pet["description"] not in readme:
            errors.append(f"grid: description missing for {pet['id']}")
        if f"docs/readme/pet-{pet['id']}.png" not in readme:
            errors.append(f"grid: image missing for {pet['id']}")
    count = readme.count(" · <em>included</em>")
    if count != 3:
        errors.append(f"grid: expected 3 included pets, found {count}")
    return errors


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    external = set()
    failed = False
    for rel in DOCS:
        errors, _ = check_doc(rel, external)
        if rel == "README.md":
            errors += grid_errors()
        if errors:
            failed = True
            print(f"FAIL {rel}")
            for error in errors:
                print(f"  {error}")
        else:
            print(f"PASS {rel}")

    print("\nEXTERNAL URLS (check by hand)")
    for url in sorted(external):
        print(url)

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())