# tempo — a multi-person worktime logger (with charts)

Everyone logs time against a **project** and a **work category** (engineering,
sales, design, …); admins create the projects and categories. Log manually or
run a live **pomodoro timer** (start → stop → an entry with the elapsed minutes).
Anyone sees their own contribution over a **week / month / year / custom range**,
broken down by project and category; **managers and admins see the whole org's
distribution**, including who contributed what. The reporting axis — group + sum
over a date range, scoped by role — is what the charts render.

Same shape as the other showcases: one **`tempo-domain`** HTTP component that
exports `wasi:http` and imports only WIT contracts — the composed **auth-guard**
(`auth:identity`) for accounts + RBAC, and **`records:store`** for the data. No
bespoke auth, no bespoke storage.

![The tempo dashboard: a manager logs in and sees the whole team's month — a donut of hours by project, bars by category and by person, and a per-day series — then flips the range (week / month / year) and the scope (Everyone / Mine), logs a live entry that updates the charts, and runs a pomodoro timer that logs an entry when stopped. A live recording of the running app.](docs/media/tempo.gif)

## Roles (RBAC via the composed auth-guard)

Three global roles, granted at register (an admin would grant them in prod):

| role | can |
|---|---|
| **member** | log time (manual or timer); see **their own** reports |
| **manager** | everything a member can, **plus** the whole org's distribution (by project, category, and **person**) |
| **admin** | create projects + categories; sees all |

Every write checks the caller's token (`authorizer::introspect`); a member asking
for `scope=all` is silently kept to `me` — you can't widen your own view.

## The data model

- **projects** / **categories** — admin-created named records.
- **entries** — `{user, project, category, minutes, day, note}`. The `day` is a
  `YYYY-MM-DD` string, so a range filter is a **string compare** (`from <= day <=
  to`) and the client owns the calendar — no server-side date math. Project and
  category names are denormalized onto the entry for fast reporting.
- **timers** — one running "pomodoro" per user; `stop` computes elapsed minutes
  and writes an entry, then deletes the timer.

## The report (what the charts read)

`GET /api/report?from&to&scope=me|all` sums minutes over the range, grouped every
way the dashboard needs, in one call:

- `by_project` — the donut + total.
- `by_category` — category bars.
- `by_day` — the per-day series.
- `matrix` — project × category (for a stacked view).
- `by_user` — per-person bars (managers/admins only).

The whole thing is exercised by `just e2e-tempo`: admin-only project/category
creation, a member logging time and being unable to escalate scope, range
filtering, a manager seeing the org grouped by user, and the pomodoro timer
producing an entry.

## Run it

```bash
just host-tempo     # native host + SPA on http://127.0.0.1:3040
# register as admin to create projects + categories; as member/manager to log + report.
just e2e-tempo      # the auth + RBAC + aggregation + timer e2e
```

## Rungs left

- **Manager assignment per project** — scope a manager to *their* projects via
  `policy:guard` (currently manager = whole-org read).
- **Edit / delete entries** and a calendar grid view (the data model already
  carries `day`).
- **Export** — CSV of a range via the `csv:codec` component.
