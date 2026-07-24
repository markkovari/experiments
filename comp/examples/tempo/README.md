# tempo — multi-person worktime logger (TEMPO.md)

Log time by project + category (or run a pomodoro timer); see your contribution
over week/month/year/custom ranges, broken down by project and category — and as
a manager, the whole team's distribution. Charts included. See
[TEMPO.md](../../TEMPO.md) for the write-up.

A composed HTTP app on the native Rust host, so this directory holds the
dashboard SPA + a Rust e2e (not a jco harness).

```
public/index.html        # the dashboard: log/timer, range + scope controls, charts
tests/tempo.rs           # e2e: auth + RBAC, logging, timer, role-scoped range reports
```

## Run

```bash
# from the repo root:
just host-tempo          # composes tempo-domain (+ auth-guard + records); SPA on :3040
```

Open `http://127.0.0.1:3040`: **register** (pick `admin` to create projects &
categories, `manager` to see the whole team, `member` to log your own). Log time
or hit **Start timer** for a pomodoro; use the range + **Everyone/Mine** controls
to drive the charts.

```bash
just e2e-tempo           # the auth + RBAC + aggregation + timer e2e (spawns the host)
```
