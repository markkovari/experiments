#!/usr/bin/env bash
# tap.sh <regex matching text, content-desc or resource-id>: taps the first matching on-screen element via uiautomator.
set -euo pipefail
adb shell uiautomator dump /sdcard/ui.xml >/dev/null
adb shell cat /sdcard/ui.xml | python3 -c '
import re, sys, xml.etree.ElementTree as ET
pat = re.compile(sys.argv[1], re.I)
for n in ET.fromstring(sys.stdin.read()).iter("node"):
    if any(pat.search(n.get(a, "")) for a in ("text", "content-desc", "resource-id")):
        x1, y1, x2, y2 = map(int, re.findall(r"\d+", n.get("bounds")))
        print((x1 + x2) // 2, (y1 + y2) // 2); sys.exit(0)
sys.exit("no element matches " + sys.argv[1])
' "$1" | { read -r x y; adb shell input tap "$x" "$y"; echo "tapped $1 at $x,$y"; }
