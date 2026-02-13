#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR/infra"
PIPELINE_DIR="$SCRIPT_DIR/pipeline"

echo "=== Code RAG Ingestion Pipeline ==="
echo ""

# 1. Start infrastructure
echo "[1/6] Starting infrastructure (SurrealDB + Ollama)..."
docker compose -f "$INFRA_DIR/docker-compose.yml" up -d --build

# 2. Wait for SurrealDB
echo "[2/6] Waiting for SurrealDB to be healthy..."
for i in $(seq 1 30); do
    if curl -sf http://localhost:8000/health > /dev/null 2>&1; then
        echo "  SurrealDB is ready."
        break
    fi
    if [ "$i" -eq 30 ]; then
        echo "  ERROR: SurrealDB did not become healthy in time."
        exit 1
    fi
    sleep 2
done

# 3. Wait for Ollama
echo "[3/6] Waiting for Ollama to be healthy..."
for i in $(seq 1 60); do
    if curl -sf http://localhost:11434/api/tags > /dev/null 2>&1; then
        echo "  Ollama is ready."
        break
    fi
    if [ "$i" -eq 60 ]; then
        echo "  ERROR: Ollama did not become healthy in time."
        exit 1
    fi
    sleep 3
done

# Ensure the embedding model is available
echo "  Ensuring nomic-embed-text model is pulled..."
curl -sf http://localhost:11434/api/tags | grep -q "nomic-embed-text" || \
    docker exec ollama ollama pull nomic-embed-text

# 4. Apply SurrealDB schema
echo "[4/6] Applying SurrealDB schema..."
curl -sf -X POST http://localhost:8000/sql \
    -H "Accept: application/json" \
    -H "NS: code_rag" \
    -H "DB: code_rag" \
    -u "root:root" \
    --data-binary "@$INFRA_DIR/surrealdb/init.surql"
echo ""
echo "  Schema applied."

# 5. Install Python dependencies
echo "[5/6] Setting up Python pipeline with uv..."
cd "$PIPELINE_DIR"
uv sync

# 6. Ingest both codebases
echo "[6/6] Ingesting codebases..."
echo ""
echo "--- Ingesting counter-service-a ---"
uv run python -m src ingest "$SCRIPT_DIR/counter-service-a" --name counter-service-a

echo ""
echo "--- Ingesting counter-service-b ---"
uv run python -m src ingest "$SCRIPT_DIR/counter-service-b" --name counter-service-b

echo ""
echo "=== Ingestion complete! ==="
echo ""
echo "Usage:"
echo "  Search:  cd $SCRIPT_DIR/pipeline && uv run python -m src search 'HTTP handler'"
echo "  Stats:   cd $SCRIPT_DIR/pipeline && uv run python -m src stats"
echo "  Cleanup: make -C $SCRIPT_DIR infra-down"
