"""CLI entry point for the code RAG ingestion pipeline."""

import asyncio

import click

from .chunker import ingest_codebase
from .embedder import embed_text
from .store import connect, search_with_graph


@click.group()
def cli():
    """Code RAG ingestion pipeline - parse, embed, and store Rust codebases."""
    pass


@cli.command()
@click.argument("codebase_path", type=click.Path(exists=True))
@click.option("--name", "-n", required=True, help="Name for this codebase")
@click.option(
    "--ollama-host",
    default="http://localhost:11434",
    help="Ollama server URL",
)
@click.option(
    "--surreal-url",
    default="ws://localhost:8000/rpc",
    help="SurrealDB WebSocket URL",
)
def ingest(codebase_path: str, name: str, ollama_host: str, surreal_url: str):
    """Ingest a Rust codebase into SurrealDB with embeddings."""
    asyncio.run(
        ingest_codebase(
            codebase_path,
            name,
            ollama_host=ollama_host,
            surreal_url=surreal_url,
        )
    )


@cli.command()
@click.argument("query")
@click.option("--limit", "-l", default=5, help="Number of results to return")
@click.option(
    "--ollama-host",
    default="http://localhost:11434",
    help="Ollama server URL",
)
@click.option(
    "--surreal-url",
    default="ws://localhost:8000/rpc",
    help="SurrealDB WebSocket URL",
)
def search(query: str, limit: int, ollama_host: str, surreal_url: str):
    """Semantic search for code chunks with graph context."""

    async def _search():
        print(f"Searching for: '{query}'")
        print(f"Embedding query...")

        query_embedding = embed_text(query, host=ollama_host)

        print(f"Querying SurrealDB...")
        client = await connect(url=surreal_url)
        results = await search_with_graph(client, query_embedding, limit=limit)

        if not results:
            print("No results found.")
            return

        # search_with_graph returns a list from client.query which may be nested
        # Flatten if needed
        items = results
        if isinstance(results, list) and len(results) > 0:
            if isinstance(results[0], list):
                items = results[-1]  # last statement result

        for i, r in enumerate(items, 1):
            if not isinstance(r, dict):
                continue
            kind = r.get("kind", "?")
            name = r.get("name", "?")
            distance = r.get("distance", "?")
            codebase = r.get("codebase_name", ["?"])
            file_path = r.get("file_path", ["?"])
            source = r.get("source_text", "")

            print(f"\n--- Result {i} ---")
            print(f"  [{kind}] {name}")
            print(f"  Codebase: {codebase}")
            print(f"  File: {file_path}")
            print(f"  Distance: {distance}")
            print(f"  Source preview:")
            for line in source.split("\n")[:10]:
                print(f"    {line}")
            if source.count("\n") > 10:
                print(f"    ... ({source.count(chr(10)) - 10} more lines)")

    asyncio.run(_search())


@cli.command()
@click.option(
    "--surreal-url",
    default="ws://localhost:8000/rpc",
    help="SurrealDB WebSocket URL",
)
def stats(surreal_url: str):
    """Show statistics about ingested data."""

    async def _stats():
        client = await connect(url=surreal_url)

        codebases = await client.query("SELECT name, path, language FROM codebase")
        files = await client.query("SELECT count() FROM file GROUP ALL")
        chunks = await client.query("SELECT count() FROM chunk GROUP ALL")
        edges = await client.query(
            """
            SELECT
                (SELECT count() FROM contains_file GROUP ALL) AS file_edges,
                (SELECT count() FROM contains_chunk GROUP ALL) AS chunk_edges,
                (SELECT count() FROM implements GROUP ALL) AS impl_edges
            """
        )

        print("=== Code RAG Statistics ===")
        print(f"\nCodebases:")
        if codebases and isinstance(codebases[0], list):
            for cb in codebases[0]:
                print(f"  - {cb.get('name')}: {cb.get('path')} ({cb.get('language')})")
        print(f"\nFiles: {files}")
        print(f"Chunks: {chunks}")
        print(f"Edges: {edges}")

    asyncio.run(_stats())


if __name__ == "__main__":
    cli()
