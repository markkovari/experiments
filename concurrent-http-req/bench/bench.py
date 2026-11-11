#!/usr/bin/env python3
"""Contention benchmark: N clients concurrently increment ONE item, per strategy.

Starts the release server on a throwaway SQLite file, runs each strategy's natural
client loop, then writes bench/results.json and bench/bench.svg.
Metrics: successful increments/s, retries per increment, lost updates.
"""
import http.client, json, os, subprocess, sys, tempfile, threading, time, uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PORT = int(os.environ.get("PORT", "3918"))
CLIENTS = [1, 4, 16, 32]
TOTAL_OPS = int(os.environ.get("OPS", "800"))  # increments per run, split across clients
STRATEGIES = ["lww", "ius", "occ", "lock", "patch", "merge", "es", "queue", "crdt"]


class Client:
    def __init__(self):
        self.c = http.client.HTTPConnection("127.0.0.1", PORT)

    def req(self, method, path, body=None, headers=None):
        h = {"content-type": "application/json", **(headers or {})}
        self.c.request(method, path, json.dumps(body) if body is not None else None, h)
        r = self.c.getresponse()
        data = r.read()
        return r.status, r.headers, (json.loads(data) if data else None)


def create_item(c):
    s, _, b = c.req("POST", "/items", {"name": "bench", "qty": 0})
    assert s == 201, s
    return b["id"]


# Each op performs one logical "+1" and returns how many retries it needed.
def op_lww(c, id):
    _, _, it = c.req("GET", f"/items/{id}")
    c.req("PUT", f"/lww/items/{id}", {"name": it["name"], "qty": it["qty"] + 1})
    return 0


def op_occ(c, id):
    retries = 0
    while True:
        _, h, it = c.req("GET", f"/items/{id}")
        s, _, _ = c.req("PUT", f"/occ/items/{id}", {"name": it["name"], "qty": it["qty"] + 1}, {"if-match": h["etag"]})
        if s == 200:
            return retries
        assert s == 412, s
        retries += 1


def op_ius(c, id):
    retries = 0
    while True:
        _, h, it = c.req("GET", f"/items/{id}")
        hdr = {"if-unmodified-since": h["last-modified"]}
        s, _, _ = c.req("PUT", f"/ius/items/{id}", {"name": it["name"], "qty": it["qty"] + 1}, hdr)
        if s == 200:
            return retries
        assert s == 412, s
        retries += 1


def op_merge(c, id):
    retries = 0
    while True:
        _, _, it = c.req("GET", f"/items/{id}")
        body = {"base_version": it["version"], "name": it["name"], "qty": it["qty"] + 1}
        s, _, _ = c.req("PUT", f"/merge/items/{id}", body)
        if s == 200:
            return retries
        assert s == 409, s  # every client changes qty, so concurrent edits truly conflict
        retries += 1


def op_crdt(c, key):
    c.p = getattr(c, "p", 0) + 1  # this client is one replica; send its running total
    s, _, _ = c.req("POST", f"/crdt/{key}", {"replica": str(id(c)), "p": c.p})
    assert s == 200, s
    return 0


def op_lock(c, id):
    retries = 0
    while True:
        s, _, lease = c.req("POST", f"/lock/items/{id}/lock")
        if s == 201:
            break
        assert s == 423, s
        retries += 1
        time.sleep(0.001)
    tok = {"lock-token": lease["token"]}
    _, _, it = c.req("GET", f"/items/{id}")
    s, _, _ = c.req("PUT", f"/lock/items/{id}", {"name": it["name"], "qty": it["qty"] + 1}, tok)
    assert s == 200, s
    c.req("DELETE", f"/lock/items/{id}/lock", None, tok)
    return retries


def op_patch(c, id):
    s, _, _ = c.req("PATCH", f"/patch/items/{id}", {"qty_inc": 1})
    assert s == 200, s
    return 0


def op_queue(c, id):
    s, _, _ = c.req("POST", f"/queue/items/{id}/commands", {"qty_inc": 1})
    assert s == 202, s
    return 0


def op_es(c, stream):
    retries = 0
    while True:
        s, _, b = c.req("GET", f"/es/streams/{stream}")
        v = b["version"] if s == 200 else 0
        s, _, _ = c.req("POST", f"/es/streams/{stream}/events", {"expected_version": v, "type": "Inc", "data": {}})
        if s == 201:
            return retries
        assert s == 409, s
        retries += 1


OPS = {"lww": op_lww, "ius": op_ius, "merge": op_merge, "crdt": op_crdt, "occ": op_occ, "lock": op_lock, "patch": op_patch, "queue": op_queue, "es": op_es}


def final_count(c, strategy, target):
    if strategy == "es":
        return c.req("GET", f"/es/streams/{target}")[2]["version"]
    if strategy == "crdt":
        return c.req("GET", f"/crdt/{target}")[2]["value"]
    return c.req("GET", f"/items/{target}")[2]["qty"]


def run(strategy, clients):
    setup = Client()
    target = f"bench-{uuid.uuid4()}" if strategy in ("es", "crdt") else create_item(setup)
    per_client = TOTAL_OPS // clients
    expected = per_client * clients
    retries = [0] * clients
    barrier = threading.Barrier(clients + 1)

    def worker(i):
        c = Client()
        barrier.wait()
        for _ in range(per_client):
            retries[i] += OPS[strategy](c, target)

    threads = [threading.Thread(target=worker, args=(i,)) for i in range(clients)]
    for t in threads:
        t.start()
    barrier.wait()
    t0 = time.perf_counter()
    for t in threads:
        t.join()
    if strategy == "queue":  # accepted != applied: wait for the worker to drain
        while final_count(setup, strategy, target) < expected:
            time.sleep(0.002)
    elapsed = time.perf_counter() - t0
    applied = final_count(setup, strategy, target)
    return {
        "strategy": strategy,
        "clients": clients,
        "ops": expected,
        "seconds": round(elapsed, 4),
        "ops_per_sec": round(expected / elapsed, 1),
        "retries_per_op": round(sum(retries) / expected, 3),
        "lost_updates": expected - applied,
    }


def plot(results, path):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    fig, axes = plt.subplots(1, 3, figsize=(16, 4.2))
    panels = [
        ("ops_per_sec", "Completed increments / s (incl. lost ones)"),
        ("retries_per_op", "Retries per increment"),
        ("lost_updates", f"Lost updates (of {TOTAL_OPS})"),
    ]
    for ax, (key, title) in zip(axes, panels):
        for s in STRATEGIES:
            rows = [r for r in results if r["strategy"] == s]
            ax.plot([r["clients"] for r in rows], [r[key] for r in rows], marker="o", label=s)
        ax.set_title(title, fontsize=11)
        ax.set_xlabel("concurrent clients")
        ax.set_xscale("log", base=2)
        ax.set_xticks(CLIENTS, [str(c) for c in CLIENTS])
        ax.grid(alpha=0.3)
    fig.legend(*axes[0].get_legend_handles_labels(), loc="center right")
    fig.suptitle("All clients increment the same item", fontsize=12)
    fig.tight_layout(rect=(0, 0, 0.92, 1))
    fig.savefig(path, format="svg")


def main():
    subprocess.run(["cargo", "build", "--release", "-q"], cwd=ROOT, check=True)
    tmp = tempfile.mkdtemp()
    env = {**os.environ, "DATABASE_URL": f"sqlite:{tmp}/bench.db", "ADDR": f"127.0.0.1:{PORT}"}
    server = subprocess.Popen([ROOT / "target/release/concurrent-http-req"], env=env, stdout=subprocess.DEVNULL)
    try:
        for _ in range(50):
            try:
                Client().req("GET", "/items/0")
                break
            except OSError:
                time.sleep(0.1)
        results = []
        for s in STRATEGIES:
            for n in CLIENTS:
                r = run(s, n)
                results.append(r)
                print(json.dumps(r), flush=True)
    finally:
        server.terminate()
    (ROOT / "bench/results.json").write_text(json.dumps(results, indent=1) + "\n")
    plot(results, ROOT / "bench/bench.svg")


if __name__ == "__main__":
    sys.exit(main())
