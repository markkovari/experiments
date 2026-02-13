# infra

Docker Compose infrastructure for the code RAG pipeline: SurrealDB and Ollama.

## Services

### SurrealDB (port 8000)

Multi-model database used as both a **vector store** and a **graph database**.

- Auth: `root` / `root`
- Storage: file-backed (`/data/database.db` inside the container)
- Namespace: `code_rag`, Database: `code_rag`

### Ollama (port 11434)

Local LLM inference server, used here for the `nomic-embed-text` embedding model (768-dimensional vectors).

- The Dockerfile pre-pulls `nomic-embed-text` at build time so it's ready immediately
- Memory: 4-8 GB reserved

## Start / Stop

```bash
docker compose up -d --build     # Start
docker compose down              # Stop (preserves data)
docker compose down -v           # Stop and wipe all data
```

## SurrealDB Schema

The schema is defined in [surrealdb/init.surql](surrealdb/init.surql) and applied by the `ingest.sh` script via the HTTP API.

### Node Tables

| Table | Fields | Purpose |
|-------|--------|---------|
| `codebase` | name, path, language | Top-level project |
| `file` | path, relative_path, language, size_bytes | Source file |
| `chunk` | kind, name, source_text, lines, bytes, signature, embedding, metadata | Code chunk with 768-dim vector |

### Vector Index

```sql
DEFINE INDEX chunk_embedding_idx ON chunk FIELDS embedding
    HNSW DIMENSION 768 DIST COSINE
    TYPE F32 EFC 150 M 12;
```

### Graph Edges (RELATION tables)

| Edge | From | To | Meaning |
|------|------|----|---------|
| `contains_file` | codebase | file | Project has source file |
| `contains_chunk` | file | chunk | File has code chunk |
| `implements` | chunk (impl) | chunk (struct/enum) | Impl block for a type |
| `calls` | chunk (fn) | chunk (fn) | Function calls function |
| `imports` | chunk (use) | chunk | Use statement imports symbol |
| `references_type` | chunk | chunk (type) | Code references a type |

### Example Queries

```sql
USE NS code_rag DB code_rag;

-- Vector similarity search
SELECT id, kind, name, vector::distance::knn() AS dist
FROM chunk
WHERE embedding <|5|> $query_vec
ORDER BY dist;

-- Graph traversal: find all chunks in a codebase
SELECT ->contains_file->file->contains_chunk->chunk
FROM codebase WHERE name = "counter-service-a";

-- Find what struct an impl block implements
SELECT ->implements->chunk.name AS type_name
FROM chunk WHERE kind = "impl";

-- Combined: similar chunks + their file and codebase context
SELECT *,
    <-contains_chunk<-file.relative_path AS file_path,
    <-contains_chunk<-file<-contains_file<-codebase.name AS codebase
FROM chunk
WHERE embedding <|5|> $query_vec
ORDER BY vector::distance::knn();
```
