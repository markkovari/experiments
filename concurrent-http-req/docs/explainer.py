#!/usr/bin/env python3
"""Renders docs/explainer.gif (+ explainer.mp4 if ffmpeg is installed):
for each strategy, the A/B race as an animated sequence diagram, plus when to use it.

    python3 docs/explainer.py
"""
import json, shutil, subprocess, tempfile, textwrap
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import FancyArrowPatch, FancyBboxPatch
from PIL import Image

DOCS = Path(__file__).resolve().parent
BENCH = DOCS.parent / "bench/results.json"

BG, PANEL, FG, DIM, LANE = "#0f172a", "#1e293b", "#e2e8f0", "#94a3b8", "#334155"
COL = {"ok": "#22c55e", "bad": "#ef4444", "warn": "#f59e0b", "info": "#94a3b8", "push": "#38bdf8"}
X = {"A": 0.12, "S": 0.5, "B": 0.88}
MONO = "DejaVu Sans Mono"

# Step: (from, to, request, response, kind). from == to draws a note on that lane.
STRATS = [
    dict(num="0", key="lww", title="Last-write-wins", idea="No guard: the last PUT replaces whatever is there.",
         steps=[("A", "S", "GET /items/1", "v1 · qty 10", "info"),
                ("B", "S", "GET /items/1", "v1 · qty 10", "info"),
                ("A", "S", "PUT qty=11", "200 · v2", "ok"),
                ("B", "S", "PUT qty=11", "200 · v3 (overwrote A)", "bad"),
                ("S", "S", "qty = 11 after two +1s: A's update is lost", "", "bad")],
         use=["Exactly one writer per resource", "Full replace where only the newest value matters (settings, presence)"],
         avoid=["Anything two people or processes can edit at the same time"]),
    dict(num="1", key="occ", title="ETag / If-Match", idea="The write must name the version it read. A write without If-Match gets 428.",
         steps=[("A", "S", "GET /items/1", 'ETag "1"', "info"),
                ("B", "S", "GET /items/1", 'ETag "1"', "info"),
                ("A", "S", 'PUT If-Match: "1"', '200 · ETag "2"', "ok"),
                ("B", "S", 'PUT If-Match: "1"', "412 Precondition Failed", "warn"),
                ("B", "S", "GET /items/1  (re-read)", 'ETag "2" · qty 11', "info"),
                ("B", "S", 'PUT If-Match: "2"  qty=12', "200", "ok")],
         use=["The default for editing REST resources", "Low to moderate contention", "Clients that can re-read and retry"],
         avoid=["One hot row with many writers: retries pile up"]),
    dict(num="2", key="ius", title="If-Unmodified-Since", idea="Like If-Match, but compares the Last-Modified time (1-second resolution).",
         steps=[("A", "S", "GET /items/1", "Last-Modified 12:00:00", "info"),
                ("B", "S", "GET /items/1", "Last-Modified 12:00:00", "info"),
                ("A", "S", "PUT If-Unmodified-Since 12:00:00", "200 (written 12:00:01)", "ok"),
                ("B", "S", "PUT If-Unmodified-Since 12:00:00", "412", "warn"),
                ("S", "S", "But writes within the same second all pass: updates get lost", "", "bad")],
         use=["Human-speed edits", "Systems that only track modification time"],
         avoid=["Fast concurrent writers"]),
    dict(num="3", key="lock", title="Lock with a lease", idea="Take an exclusive, expiring lock before writing.",
         steps=[("A", "S", "POST /items/1/lock", "201 · token t1", "ok"),
                ("B", "S", "POST /items/1/lock", "423 Locked", "warn"),
                ("A", "S", "PUT  Lock-Token: t1", "200", "ok"),
                ("A", "S", "DELETE /items/1/lock", "204", "info"),
                ("B", "S", "POST /items/1/lock", "201 · token t2", "ok"),
                ("S", "S", "If A crashes, its lease expires and B can take over", "", "info")],
         use=["Long edits (a document open in an editor)", "Conflicts that are expensive or impossible to merge",
              "Work with external side effects"],
         avoid=["High throughput: 3 extra round-trips per write", "Clients that hold locks too long"]),
    dict(num="4", key="patch", title="Partial + relative PATCH", idea="Send only what changed, as a delta. The DB runs qty = qty + 1 atomically.",
         steps=[("A", "S", 'PATCH {"name": "A"}', "200", "ok"),
                ("B", "S", 'PATCH {"qty_inc": 1}', "200", "ok"),
                ("A", "S", 'PATCH {"qty_inc": 1}', "200", "ok"),
                ("S", "S", "name = A, qty = 12: every change kept, no read needed", "", "ok")],
         use=["Counters, stock, likes", "Independent fields edited by different people", "Fastest in the benchmark"],
         avoid=["Absolute values with rules across fields (add expected_version for those)"]),
    dict(num="5", key="merge", title="Three-way merge", idea="Send the version you started from; the server merges field by field.",
         steps=[("A", "S", "GET /items/1", "v1 · name widget · qty 10", "info"),
                ("B", "S", "GET /items/1", "v1 · name widget · qty 10", "info"),
                ("A", "S", "PUT base=1 name=A", "200 · v2", "ok"),
                ("B", "S", "PUT base=1 qty=15", "200 · name A, qty 15 (merged)", "ok"),
                ("B", "S", "PUT base=1 name=B  (B renamed too)", "409 · conflict on name", "warn")],
         use=["Forms and documents with many independent fields", "Offline edits synced later",
              "Showing users exactly what conflicts"],
         avoid=["Counters: two +1s look like the same change"]),
    dict(num="6", key="es", title="Event sourcing", idea="Store events, not state. Append only if you saw the latest version.",
         steps=[("A", "S", "POST event  expected=0", "201 · v1", "ok"),
                ("B", "S", "POST event  expected=0", "409 · current_version 1", "warn"),
                ("B", "S", "GET stream  (catch up)", "1 event", "info"),
                ("B", "S", "POST event  expected=1", "201 · v2", "ok"),
                ("S", "S", "PRIMARY KEY (stream, version) blocks a second append at v1", "", "info")],
         use=["Audit trails and full history", "Events other services consume", "Rebuilding read models"],
         avoid=["Simple CRUD", "One hot stream with many writers (slowest in the benchmark)"]),
    dict(num="7", key="queue", title="Command queue", idea="Writes become commands; one worker applies them in order.",
         steps=[("A", "S", 'POST /commands {"qty_inc": 1}', "202 · /commands/1", "ok"),
                ("B", "S", 'POST /commands {"qty_inc": 1}', "202 · /commands/2", "ok"),
                ("S", "S", "worker applies #1, then #2: never in parallel", "", "info"),
                ("A", "S", "GET /commands/1", "done", "ok"),
                ("B", "S", "GET /commands/2", "done", "ok")],
         use=["Slow or asynchronous work", "Strict ordering per resource", "Smoothing bursts of writes"],
         avoid=["Clients that need the result immediately"]),
    dict(num="8", key="crdt", title="CRDT", idea="Data types whose merges converge in any order: each replica sends its running totals.",
         steps=[("A", "A", "offline: +1 +1 +1", "", "info"),
                ("A", "S", "POST {replica a, p 3}", "200 · value 3", "ok"),
                ("B", "S", "POST {replica b, p 2}", "200 · value 5", "ok"),
                ("A", "S", "POST {replica a, p 3}  (resent)", "200 · value 5, no double count", "ok"),
                ("S", "S", "value = sum of each replica's max total", "", "info")],
         use=["Offline-first apps", "Multi-region without coordination", "Counters, sets, presence"],
         avoid=['Rules like "stock never below 0"', "Complex domain logic"]),
    dict(num="9", key="ot", title="Operational transform", idea="Rewrite each edit against the edits it didn't see.",
         steps=[("S", "S", 'doc v0: "hello world"', "", "info"),
                ("A", "S", 'insert(0, "Hey, ")  base 0', '201 · "Hey, hello world"', "ok"),
                ("B", "S", "delete(6, 5)  base 0", "shifted to delete(11, 5)", "ok"),
                ("S", "S", '"Hey, hello ": both edits kept as intended', "", "ok")],
         use=["Real-time collaborative text (Google Docs style)"],
         avoid=["Anything that isn't text", "Building it yourself: prefer a text CRDT library (Yjs, Automerge)"]),
    dict(num="+", key="sse", title="Change notifications (SSE)", idea="Push every new version so clients edit fresh data.",
         steps=[("B", "S", "GET /items/1/events", "stream open", "info"),
                ("A", "S", 'PATCH {"qty_inc": 1}', "200 · v2", "ok"),
                ("S", "B", "event: item  v2", "", "push"),
                ("B", "S", 'PUT If-Match: "2"', "200", "ok"),
                ("S", "S", "Shrinks the conflict window; still pair it with #1 or #4", "", "info")],
         use=["Collaborative UIs and dashboards", "Alongside any strategy above"],
         avoid=["Relying on it alone: a write can still race the push"]),
]

GUIDE = [
    ("Can the change be a delta (+1, set one field)?", "#4 PATCH"),
    ("Normal REST edit, a few concurrent writers?", "#1 ETag / If-Match"),
    ("Long human edit, conflicts are expensive?", "#3 Lock lease"),
    ("Many fields, offline forms, show the conflict?", "#5 Three-way merge"),
    ("Need full history or an audit trail?", "#6 Event sourcing"),
    ("Need ordering or async processing?", "#7 Command queue"),
    ("Offline or multi-region, no coordination?", "#8 CRDT"),
    ("Real-time collaborative text?", "#9 OT / text CRDT"),
    ("Want fewer conflicts with any of the above?", "+ SSE"),
]


def bench_line(key):
    try:
        rows = json.loads(BENCH.read_text())
    except OSError:
        return None
    r = next((r for r in rows if r["strategy"] == key and r["clients"] == 32), None)
    if not r:
        return None
    return f"{r['ops_per_sec']:.0f} inc/s · {r['retries_per_op']:.1f} retries/inc\n{r['lost_updates']} of {r['ops']} increments lost"


def new_fig():
    return plt.figure(figsize=(12.8, 7.2), dpi=100, facecolor=BG)


def progress(fig, current):
    labels = [s["num"] for s in STRATS]
    for i, n in enumerate(labels):
        x = 0.31 + i * 0.035
        on = n == current
        fig.patches.append(FancyBboxPatch((x, 0.025), 0.026, 0.04, boxstyle="round,pad=0.003",
                                          transform=fig.transFigure, fc="#38bdf8" if on else PANEL, ec="none"))
        fig.text(x + 0.013, 0.045, n, ha="center", va="center", color=BG if on else DIM, size=11, weight="bold")


def wrap(text, width):
    return "\n".join(textwrap.wrap(text, width))


def scene(s, shown):
    fig = new_fig()
    fig.text(0.035, 0.925, f"#{s['num']}  {s['title']}", color=FG, size=26, weight="bold")
    fig.text(0.035, 0.875, s["idea"], color=DIM, size=14)

    ax = fig.add_axes([0.02, 0.1, 0.62, 0.74])
    ax.set_xlim(0, 1), ax.set_ylim(0, 1), ax.axis("off")
    for lane, name in [("A", "Client A"), ("S", "Server + SQLite"), ("B", "Client B")]:
        ax.add_patch(FancyBboxPatch((X[lane] - 0.1, 0.925), 0.2, 0.06, boxstyle="round,pad=0.01", fc=PANEL, ec=LANE))
        ax.text(X[lane], 0.955, name, ha="center", va="center", color=FG, size=12, weight="bold")
        ax.plot([X[lane]] * 2, [0.02, 0.91], color=LANE, lw=1.5, ls=(0, (4, 4)))

    for i, (frm, to, req, res, kind) in enumerate(s["steps"][:shown]):
        y = 0.83 - i * 0.135
        c = COL[kind]
        if frm == to:  # note
            ax.text(X[frm] if frm != "S" else 0.5, y - 0.02, wrap(req, 52), ha="center", va="center", color=c,
                    size=11.5, weight="bold", bbox=dict(boxstyle="round,pad=0.45", fc=BG, ec=c, lw=1.5))
            continue
        x0, x1 = X[frm], X[to]
        ax.add_patch(FancyArrowPatch((x0, y), (x1, y), arrowstyle="-|>", mutation_scale=18, color=c, lw=2.2))
        mid = (x0 + x1) / 2
        ax.text(mid, y + 0.022, req, ha="center", va="bottom", color=FG, size=10.5, family=MONO)
        if res:
            ax.text(mid, y - 0.022, "→ " + res, ha="center", va="top", color=c, size=10.5, family=MONO, weight="bold")

    side = fig.add_axes([0.665, 0.1, 0.31, 0.74])
    side.set_xlim(0, 1), side.set_ylim(0, 1), side.axis("off")
    side.add_patch(FancyBboxPatch((0, 0), 1, 1, boxstyle="round,pad=0.0,rounding_size=0.03", fc=PANEL, ec="none"))
    y = 0.93
    for head, items, c in [("Use when", s["use"], COL["ok"]), ("Avoid when", s["avoid"], COL["bad"])]:
        side.text(0.06, y, head, color=c, size=15, weight="bold", va="top")
        y -= 0.075
        for it in items:
            t = wrap(it, 34)
            side.text(0.06, y, "•  " + t.replace("\n", "\n    "), color=FG, size=12, va="top", linespacing=1.35)
            y -= 0.058 * (t.count("\n") + 1) + 0.025
        y -= 0.04
    b = bench_line(s["key"])
    if b:
        side.text(0.06, 0.13, "Benchmark: 32 clients, one item", color=DIM, size=11, va="top", weight="bold")
        side.text(0.06, 0.075, b, color=FG, size=11, va="top", family=MONO)
    progress(fig, s["num"])
    return fig


def intro():
    fig = new_fig()
    fig.text(0.5, 0.72, "Concurrent edits over HTTP", ha="center", color=FG, size=40, weight="bold")
    fig.text(0.5, 0.62, "Two clients read the same item, then both write. Whose change survives?",
             ha="center", color=DIM, size=18)
    names = [f"#{s['num']} {s['title']}" for s in STRATS]
    for i, n in enumerate(names):
        col, row = divmod(i, 6)
        fig.text(0.2 + col * 0.36, 0.48 - row * 0.06, n, color=FG, size=15, family=MONO)
    fig.text(0.5, 0.06, "Axum + SQLite · every scene is a Hurl test in tests/", ha="center", color=DIM, size=12)
    return fig


def outro():
    fig = new_fig()
    fig.text(0.05, 0.88, "Which one?", color=FG, size=32, weight="bold")
    for i, (q, a) in enumerate(GUIDE):
        y = 0.77 - i * 0.068
        fig.text(0.05, y, q, color=FG, size=16)
        fig.text(0.66, y, "→  " + a, color=COL["push"], size=16, weight="bold", family=MONO)
    fig.text(0.05, 0.08, "Avoid #0 (only safe with one writer) and #2 (blind within the same second).",
             color=COL["bad"], size=15, weight="bold")
    return fig


def frames():
    """Yields (figure, seconds on screen)."""
    yield intro(), 5
    for s in STRATS:
        n = len(s["steps"])
        for shown in range(n + 1):
            yield scene(s, shown), 1.5 if shown < n else 6
    yield outro(), 12


def main():
    tmp = Path(tempfile.mkdtemp())
    images, durations = [], []
    for i, (fig, secs) in enumerate(frames()):
        path = tmp / f"{i:03}.png"
        fig.savefig(path, facecolor=BG)
        plt.close(fig)
        images.append(path)
        durations.append(secs)

    gif = [Image.open(p).convert("RGB").quantize(colors=64, method=Image.Quantize.MEDIANCUT) for p in images]
    gif[0].save(DOCS / "explainer.gif", save_all=True, append_images=gif[1:], loop=0,
                duration=[int(d * 1000) for d in durations], optimize=True)
    print(f"wrote docs/explainer.gif ({len(images)} frames, {sum(durations):.0f} s)")

    if shutil.which("ffmpeg"):
        concat = tmp / "list.txt"
        lines = [f"file '{p}'\nduration {d}" for p, d in zip(images, durations)]
        concat.write_text("\n".join(lines) + f"\nfile '{images[-1]}'\n")  # concat demuxer needs the last file twice
        subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", concat,
                        "-vf", "fps=10,format=yuv420p", "-c:v", "libx264", "-crf", "23", DOCS / "explainer.mp4"],
                       check=True)
        print("wrote docs/explainer.mp4")
    shutil.rmtree(tmp)


if __name__ == "__main__":
    main()
