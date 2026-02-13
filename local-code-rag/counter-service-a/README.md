# counter-service-a

A wasmCloud component that implements an HTTP counter backed by a WASI key-value store.

## What It Does

Handles HTTP requests and uses the request path as a key-value counter key. Each request atomically increments the counter and returns the new value.

```
GET /foo  ->  Counter 'foo': 1
GET /foo  ->  Counter 'foo': 2
GET /bar  ->  Counter 'bar': 1
```

## WASI Interfaces

| Interface | Direction | Purpose |
|-----------|-----------|---------|
| `wasi:http/incoming-handler@0.2.2` | exported | Receives HTTP requests |
| `wasi:keyvalue/store` | imported | Opens a KV bucket |
| `wasi:keyvalue/atomics` | imported | Atomic counter increment |

## Code Structure

```
counter-service-a/
├── .cargo/config.toml    # Build target: wasm32-wasip2
├── Cargo.toml            # Dependencies: wasmcloud-component
├── wasmcloud.toml        # wasmCloud project config
├── wadm.yaml             # Deployment manifest (HTTP server + Redis KV)
├── wit/world.wit         # WIT world definition
└── src/lib.rs            # Component implementation
```

This service uses a straightforward, single-struct style: the `Counter` struct implements `http::Server` with all logic inline in the `handle` method.

## Build and Run

```bash
# Prerequisites
rustup target add wasm32-wasip2
cargo install wash-cli

# Build the component
wash build

# Run in development mode (starts HTTP server + in-memory KV automatically)
wash dev

# Test
curl http://localhost:8080/hello
```

## WADM Deployment

The [wadm.yaml](wadm.yaml) manifest deploys with:
- **httpserver** provider on port 8080
- **kvredis** provider connecting to Redis at `redis://0.0.0.0:6379`

For production, replace the `file://` image reference with an OCI registry URL.
