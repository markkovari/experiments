# Local Code RAG

A proof-of-concept pipeline that ingests codebases into a combined vector + graph database for code-aware Retrieval-Augmented Generation (RAG).

## What This Is

This project demonstrates an end-to-end workflow for building a code knowledge base:

1. **Two wasmCloud HTTP services** (Rust) serve as target codebases to ingest. Both implement the same thing -- an HTTP counter backed by a WASI key-value store -- but with different code structures, giving the pipeline varied source material.

2. **A Python ingestion pipeline** that:
   - Parses **Rust source files** using **tree-sitter** for AST-aware chunking (functions, structs, impls, traits, methods, etc.)
   - Parses **project descriptors** -- `Cargo.toml`, `wasmcloud.toml` (TOML sections), `wadm.yaml` (YAML top-level keys), `world.wit` (WIT worlds, interfaces, exports/imports)
   - Embeds each chunk locally using **Ollama** with the `nomic-embed-text` model (768-dimensional vectors)
   - Stores everything in **SurrealDB**, used simultaneously as:
     - A **vector database** (HNSW index for semantic similarity search)
     - A **graph database** (RELATE edges: codebase -> files -> chunks, impl -> struct, etc.)

3. **Infrastructure** via Docker Compose (SurrealDB + Ollama) and a single startup script.

## Architecture

```
                          +-------------------+
                          |  counter-service-a |  .rs, Cargo.toml, wadm.yaml,
                          |  counter-service-b |  wasmcloud.toml, world.wit
                          +---------+---------+
                                    |
                           tree-sitter (source)
                           section split (config)
                                    |
                                    v
                          +-------------------+
                          |   Code Chunks     |  functions, structs, impls,
                          |   (AST + config)  |  TOML tables, YAML keys,
                          |                   |  WIT worlds/interfaces...
                          +---------+---------+
                                    |
                           Ollama embed (local)
                           nomic-embed-text
                                    |
                                    v
                          +-------------------+
                          |    SurrealDB      |
                          |                   |
                          |  Vector: HNSW idx |  semantic similarity search
                          |  Graph: RELATE    |  codebase->file->chunk->impl->struct
                          +-------------------+
                                    |
                                    v
                          +-------------------+
                          |  CLI: search      |  "find code similar to X"
                          |       stats       |  + graph context (file, codebase,
                          |                   |    related types, implementations)
                          +-------------------+
```

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [uv](https://docs.astral.sh/uv/) (Python package manager)
- [wash](https://wasmcloud.com/docs/installation) (wasmCloud CLI, only needed to build/run the services)
- [Rust](https://rustup.rs/) with `wasm32-wasip2` target (only needed to build the services)

## Quick Start

The fastest way to get everything running:

```bash
./ingest.sh
```

This single script will:
1. Start SurrealDB and Ollama via Docker Compose
2. Wait for both services to be healthy
3. Ensure the `nomic-embed-text` embedding model is available
4. Apply the SurrealDB schema (tables, vector index, graph edges)
5. Install Python dependencies with `uv`
6. Parse and ingest both codebases (source files + project descriptors)

## Step-by-Step Usage

### 1. Start Infrastructure

```bash
make infra-up
```

This launches SurrealDB (port 8000) and Ollama (port 11434). The Ollama image pre-pulls `nomic-embed-text` at build time.

### 2. Build the wasmCloud Services (Optional)

The services don't need to be built for the pipeline to work -- the pipeline reads source and config files directly (`.rs`, `.toml`, `.yaml`, `.wit`). But if you want to actually run them:

```bash
# Install the wasm target
rustup target add wasm32-wasip2

make build
```

Then run either service with `wash dev` (see each service's README).

### 3. Ingest Codebases

```bash
make ingest
```

Or manually:

```bash
cd pipeline
uv run python -m src ingest ../counter-service-a --name counter-service-a
uv run python -m src ingest ../counter-service-b --name counter-service-b
```

### 4. Search

```bash
cd pipeline
uv run python -m src search "HTTP request handler"
uv run python -m src search "key-value increment"
uv run python -m src search "error handling pattern"
```

Search returns semantically similar code chunks along with graph context: which file and codebase they belong to, what types they implement, etc.

### 5. View Stats

```bash
cd pipeline
uv run python -m src stats
```

### 6. Tear Down

```bash
make clean        # Stop containers, remove volumes, delete venv and build artifacts
# or just:
make infra-down   # Stop containers only (preserves data)
```

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make all` | Alias for `make ingest` |
| `make infra-up` | Start SurrealDB + Ollama |
| `make infra-down` | Stop SurrealDB + Ollama |
| `make infra-reset` | Wipe volumes and restart |
| `make build` | Build both wasmCloud services |
| `make build-a` | Build counter-service-a |
| `make build-b` | Build counter-service-b |
| `make setup` | Install Python deps with uv |
| `make ingest` | Run full ingestion pipeline |
| `make search` | Interactive semantic search |
| `make stats` | Show ingestion statistics |
| `make clean` | Remove everything |

## Project Structure

```
local-code-rag/
├── counter-service-a/      # wasmCloud component A (canonical style)
├── counter-service-b/      # wasmCloud component B (refactored style)
├── pipeline/               # Python ingestion pipeline
├── infra/                  # Docker Compose + SurrealDB schema
├── ingest.sh               # One-shot setup + ingestion script
└── Makefile                # Convenience targets
```

See each subdirectory's README for details.

## How the Graph Works

SurrealDB stores both vectors and graph edges in the same database. After ingestion, you can traverse relationships like:

```
codebase:service-a
  ├──[contains_file]──> file:Cargo.toml
  │                       └──[contains_chunk]──> chunk:[package] (toml_table)
  │                       └──[contains_chunk]──> chunk:[dependencies] (toml_table)
  ├──[contains_file]──> file:wadm.yaml
  │                       └──[contains_chunk]──> chunk:spec (yaml_mapping)
  ├──[contains_file]──> file:wit/world.wit
  │                       └──[contains_chunk]──> chunk:counter (wit_world)
  │                       └──[contains_chunk]──> chunk:export wasi:http/... (wit_function)
  └──[contains_file]──> file:src/lib.rs
                          └──[contains_chunk]──> chunk:Counter (struct)
                          └──[contains_chunk]──> chunk:Counter (impl)
                                                   └──[implements]──> chunk:Counter (struct)
                          └──[contains_chunk]──> chunk:handle (method)
```

The `search` command combines vector similarity with graph traversal: it finds the most semantically similar chunks, then walks the graph to include the file path, codebase name, and related types in the results.
