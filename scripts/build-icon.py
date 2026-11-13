#!/usr/bin/env python3
import base64
import pathlib

root = pathlib.Path(__file__).resolve().parent.parent
svg = (root / "brand" / "coin.svg").read_bytes()
uri = "data:image/svg+xml;base64," + base64.b64encode(svg).decode("ascii")
target = root / "contract" / "src" / "icon.txt"
target.write_text(uri)
print(f"wrote {target.relative_to(root)}: {len(uri)} characters from {len(svg)} bytes of SVG")
