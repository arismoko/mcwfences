#!/usr/bin/env python3
"""Convert pre-26.3 loot tables ("condition"/"function" keys) to 26.3 "type" keys."""
import json
import pathlib
import sys


def convert(node):
    if isinstance(node, list):
        return [convert(v) for v in node]
    if not isinstance(node, dict):
        return node
    out = {}
    for key, value in node.items():
        value = convert(value)
        if key in ("condition", "function") and isinstance(value, str) and "type" not in node:
            key = "type"
        out[key] = value
    return out


def main():
    root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    changed = 0
    for path in sorted(root.glob("data/*/loot_table/**/*.json")):
        text = path.read_text()
        new = json.dumps(convert(json.loads(text)), indent=2) + "\n"
        if new != text:
            path.write_text(new)
            changed += 1
    print(f"rewrote {changed} files")


if __name__ == "__main__":
    main()
