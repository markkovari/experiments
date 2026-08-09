"""Assert what each fixture should do, and say plainly which one failed.

Positives are checked by INVOKING them, not by reading inventory: an app that is
placed but does not answer is exactly the failure a status check would miss.

Negatives are checked by the reason the reconciler gave. Asserting only "it was
refused" would pass for a refusal with the wrong reason, and a reason nobody can act
on is barely better than a crash.
"""
import json
import subprocess
import sys
import time
import urllib.request

sp = sys.argv[1]
INGRESS = "http://127.0.0.1:8094/api/ratelimit"
BODY = {"key": "e2e", "capacity": 10**8, "refill": 10**8}

# host -> what a request to it must produce
SERVES = {
    "fused.e2e.test": "a fused artifact serves over HTTP",
    "linked.e2e.test": "a runtime-linked graph serves, so both imports were bound",
    "zero.e2e.test": "a scaled-to-zero app is activated by the request itself",
}
# app -> substrings its refusal must contain
REFUSED = {
    "conflict": ["records:store/store", "record-store", "shaper"],
    "ungrantable": ["wasi:blobstore/blobstore"],
    "unplaceable": ["region", "mars"],
}


def invoke(host, timeout=30):
    """Retry until it serves or the deadline passes. A cold app has to be activated,
    and placement is a heartbeat behind reality — polling is the honest way to ask."""
    deadline = time.time() + timeout
    last = None
    while time.time() < deadline:
        req = urllib.request.Request(
            INGRESS, data=json.dumps(BODY).encode(),
            headers={"content-type": "application/json", "Host": host})
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                if r.status == 200:
                    return True, r.status
                last = r.status
        except Exception as e:
            last = getattr(e, "code", str(e)[:40])
        time.sleep(1)
    return False, last


def reasons():
    """What the reconciler said it could not place, keyed by app."""
    out = {}
    try:
        text = open(f"{sp}/rec.log").read()
    except FileNotFoundError:
        return out
    for line in text.splitlines():
        if "unschedulable:" not in line:
            continue
        head, _, why = line.partition("unschedulable:")
        app = head.strip().split()[-1]
        out.setdefault(app.split("/")[-1], set()).add(why.strip())
    return out


failures = []

print("=== apps that must serve ===")
for host, what in SERVES.items():
    ok, code = invoke(host)
    print(f"    {'PASS' if ok else 'FAIL'}  {host:22} {what}"
          + ("" if ok else f"  (last: {code})"))
    if not ok:
        failures.append(f"{host} never served (last response {code})")

print()
print("=== apps that must be refused, with a usable reason ===")
said = reasons()
for app, expected in REFUSED.items():
    why = " | ".join(sorted(said.get(app, [])))
    missing = [e for e in expected if e not in why]
    ok = bool(why) and not missing
    print(f"    {'PASS' if ok else 'FAIL'}  {app:12} {why[:90] if why else 'NOT REFUSED AT ALL'}")
    if not ok:
        failures.append(
            f"{app}: refusal missing {missing}" if why else f"{app} was not refused")

print()
# The property that only a shared fleet can show: one bad manifest must not stop the
# good ones. Three refusals above with three apps serving is that assertion.
served_count = sum(1 for h in SERVES if invoke(h, timeout=5)[0])
print(f"    {served_count}/{len(SERVES)} apps still serving alongside {len(REFUSED)} refused ones")
if served_count != len(SERVES):
    failures.append("a refused manifest interfered with healthy apps")

print()
if failures:
    print("FAIL")
    for f in failures:
        print(f"  - {f}")
    sys.exit(1)
print("PASS — six fixtures, three served, three refused for the right reasons")
