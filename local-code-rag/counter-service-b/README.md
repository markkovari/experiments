# counter-service-b

A wasmCloud component implementing the same HTTP counter as `counter-service-a`, but with a different code structure.

## What It Does

Identical functionality to service A -- HTTP counter backed by WASI KV -- but returns JSON and uses a refactored code layout:

```
GET /foo  ->  {"key": "foo", "count": 1}
GET /foo  ->  {"key": "foo", "count": 2}
```

## How It Differs from Service A

This service exists to give the ingestion pipeline two structurally different codebases. The differences:

| Aspect | Service A | Service B |
|--------|-----------|-----------|
| Struct name | `Counter` | `KvCounter` |
| Code layout | All inline in `handle` | Factored into helpers |
| Modules | Single file | `lib.rs` + `helpers.rs` |
| Response format | Plain text | JSON |
| Error messages | Generic | Descriptive with key names |
| Key extraction | Inline | `extract_key()` helper |
| Counter logic | Inline | `increment_counter()` function |

## Code Structure

```
counter-service-b/
├── .cargo/config.toml
├── Cargo.toml
├── wasmcloud.toml
├── wadm.yaml             # Deploys on port 8081
├── wit/world.wit
└── src/
    ├── lib.rs            # KvCounter struct + handle + increment_counter
    └── helpers.rs        # extract_key(), format_response(), tests
```

## Build and Run

```bash
rustup target add wasm32-wasip2
cargo install wash-cli

wash build
wash dev

curl http://localhost:8081/hello
```
