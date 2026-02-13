# pipeline

Python ingestion pipeline that parses codebases with tree-sitter and section-based splitting, embeds chunks via Ollama, and stores them in SurrealDB as both vectors and a graph.

## How It Works

### 1. Parse

The pipeline ingests all relevant files from a codebase, using different parsing strategies depending on the file type.

**Source files (tree-sitter AST parsing):**

`.rs` files are parsed with the [tree-sitter Rust grammar](https://github.com/tree-sitter/tree-sitter-rust), extracting:

| Chunk Kind | Example |
|-----------|---------|
| `function` | `fn increment_counter(key: &str) -> ...` |
| `struct` | `struct Counter;` |
| `enum` | `enum Mode { ... }` |
| `impl` | `impl http::Server for Counter { ... }` |
| `method` | Methods inside impl blocks (extracted as separate chunks) |
| `trait` | `trait Handler { ... }` |
| `use` | `use wasmcloud_component::http;` |
| `mod` | `mod helpers;` |
| `const` / `static` | `const MAX: u32 = 100;` |
| `macro` | `macro_rules! my_macro { ... }` |

For `impl` blocks, the parser also extracts individual methods as child chunks, so both the full impl and each method get their own embeddings.

**Project descriptors (section-based splitting):**

| File Type | Extensions | Chunk Kinds | Split Strategy |
|-----------|-----------|-------------|---------------|
| TOML | `.toml` | `config_file`, `toml_table`, `toml_section` | By `[table]` headers |
| YAML | `.yaml`, `.yml` | `config_file`, `yaml_mapping` | By top-level keys |
| WIT | `.wit` | `config_file`, `wit_world`, `wit_interface`, `wit_function`, `wit_type` | By `world`/`interface` blocks + inner declarations |

Every config file also gets a `config_file` chunk containing the entire file for full-context embedding.

**Files ingested per wasmCloud service:**
- `src/lib.rs` (and any other `.rs` files)
- `Cargo.toml` -- package metadata, dependencies
- `wasmcloud.toml` -- wasmCloud project config
- `wadm.yaml` -- deployment manifest (components, providers, links)
- `wit/world.wit` -- WASI interface definitions

Build artifacts (`target/`, `build/`) are automatically skipped.

### 2. Embed (Ollama)

Each chunk is embedded using `nomic-embed-text` (768 dimensions) via a local Ollama instance. The embedding input is language-aware:

| Format | Prefix |
|--------|--------|
| Rust | `Rust {kind} '{name}' in {file}:` |
| TOML | `TOML configuration '{name}' in {file}:` |
| YAML | `YAML manifest '{name}' in {file}:` |
| WIT | `WIT interface definition '{name}' in {file}:` |

Chunks are embedded in batches of 32 for efficiency.

### 3. Store (SurrealDB)

Chunks are stored as records in SurrealDB with an HNSW vector index for similarity search. Graph edges are created using `RELATE`:

```
codebase --[contains_file]--> file --[contains_chunk]--> chunk
chunk (impl) --[implements]--> chunk (struct/enum)
```

### 4. Search

Semantic search finds the K nearest chunks by cosine distance, then walks the graph to enrich results with file paths, codebase names, and related types.

## Setup

Requires [uv](https://docs.astral.sh/uv/):

```bash
uv sync
```

## CLI Commands

```bash
# Ingest a codebase
uv run python -m src ingest <path-to-codebase> --name <name>

# Semantic search with graph context
uv run python -m src search "HTTP request handler" --limit 5
uv run python -m src search "wasmcloud component dependencies"
uv run python -m src search "WASI keyvalue interface export"

# Show statistics
uv run python -m src stats
```

### Options

All commands accept:
- `--ollama-host` (default: `http://localhost:11434`)
- `--surreal-url` (default: `ws://localhost:8000/rpc`)

## Module Overview

| File | Purpose |
|------|---------|
| [main.py](src/main.py) | CLI entry point (click) |
| [parser.py](src/parser.py) | Tree-sitter Rust parsing and chunk extraction |
| [config_parser.py](src/config_parser.py) | TOML, YAML, and WIT section-based parsing |
| [embedder.py](src/embedder.py) | Ollama embedding client (batch support) |
| [store.py](src/store.py) | SurrealDB vector + graph storage and search |
| [chunker.py](src/chunker.py) | Orchestration: collect files, route to parsers, embed, store, relate |
| [models.py](src/models.py) | Dataclasses: `CodeChunk`, `FileInfo`, `CodebaseInfo` |

## Dependencies

- `tree-sitter` + `tree-sitter-rust` -- AST parsing for Rust source files
- `ollama` -- embedding client
- `surrealdb` -- database client (async, WebSocket)
- `click` -- CLI framework
- `httpx` -- HTTP client
