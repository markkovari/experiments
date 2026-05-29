# zig-state-machine — clean-architecture todo (Zig)

A todo API built as a lesson in Zig sub-packages (modules) + clean architecture.
Each layer is its own Zig module wired in `build.zig`; dependencies point inward
and missing `addImport`s are hard walls between layers.

## Layers

```
domain     pure Todo state machine (todo -> in_progress -> done). std only.
usecases   app logic + TodoRepo interface (hand-built vtable) + Stored(T).  -> domain
memory     in-memory TodoRepo impl.                                          -> usecases
sqlite     sqlite-backed TodoRepo impl (vendored amalgamation, extern FFI).  -> usecases
http       request -> use case -> JSON, over std.http.Server.               -> usecases
main       composition root: picks a backend from env, runs the accept loop.
```

Only `main` sees concrete repos. Swapping `memory` <-> `sqlite` is one branch in
`main`; `domain`/`usecases`/`http` never change. That is the whole point.

## Config (12-factor: all via environment)

| Var            | Default      | Meaning                                  |
|----------------|--------------|------------------------------------------|
| `TODO_BACKEND` | `sqlite`     | `memory` (ephemeral) or `sqlite` (disk)  |
| `TODO_DB_PATH` | `todos.db`   | sqlite file (ignored for memory)         |
| `HOST`         | `0.0.0.0`    | bind address                             |
| `PORT`         | `8080`       | listen port                              |

## Run

```sh
zig build run                              # sqlite, ./todos.db, :8080
TODO_BACKEND=memory PORT=9000 zig build run
```

## API

```
POST /todos                {"title":"..."}  -> 201 {id,title,status}
GET  /todos                                 -> 200 [ ... ]
POST /todos/{id}/start                      -> 200 (409 if illegal)
POST /todos/{id}/complete                   -> 200 (404 if missing)
```

## Test

```sh
zig build test    # unit tests for every module (incl. sqlite :memory:)
zig build e2e     # spawns the real binary, drives it over TCP, asserts
```

## Docker

Multi-stage build → fully static musl binary on a tiny Alpine runtime.

```sh
docker compose up todo          # sqlite, persisted to ./data
docker compose up todo-memory   # in-memory, ephemeral
```

Note: the build pins a specific Zig **dev** tarball (this project uses
0.17.0-dev-only stdlib API). Dev tarballs are transient — if the build 404s,
bump `ZIG_VERSION` in the `Dockerfile` to a current dev build.
