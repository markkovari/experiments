from surrealdb import AsyncSurreal

from .models import CodeChunk, FileInfo, CodebaseInfo

SURREAL_URL = "ws://localhost:8000/rpc"
SURREAL_NS = "code_rag"
SURREAL_DB = "code_rag"


async def connect(
    url: str = SURREAL_URL, ns: str = SURREAL_NS, db: str = SURREAL_DB
) -> AsyncSurreal:
    client = AsyncSurreal(url)
    await client.connect()
    await client.signin({"username": "root", "password": "root"})
    await client.use(ns, db)
    return client


async def store_codebase(client: AsyncSurreal, codebase: CodebaseInfo):
    """Create a codebase node. Returns the record ID."""
    result = await client.create(
        "codebase",
        {
            "name": codebase.name,
            "path": codebase.path,
            "language": codebase.language,
        },
    )
    return result["id"]


async def store_file(client: AsyncSurreal, file_info: FileInfo, codebase_id):
    """Create a file node and relate it to its codebase."""
    result = await client.create(
        "file",
        {
            "path": file_info.path,
            "relative_path": file_info.relative_path,
            "language": file_info.language,
            "size_bytes": file_info.size_bytes,
        },
    )
    file_id = result["id"]

    await client.query(
        "RELATE $cb->contains_file->$f",
        {"cb": codebase_id, "f": file_id},
    )
    return file_id


async def store_chunk(client: AsyncSurreal, chunk: CodeChunk, file_id):
    """Create a chunk node with embedding and relate it to its file."""
    result = await client.create(
        "chunk",
        {
            "kind": chunk.kind,
            "name": chunk.name,
            "source_text": chunk.source_text,
            "start_line": chunk.start_line,
            "end_line": chunk.end_line,
            "start_byte": chunk.start_byte,
            "end_byte": chunk.end_byte,
            "signature": chunk.signature,
            "embedding": chunk.embedding,
            "metadata": chunk.metadata,
        },
    )
    chunk_id = result["id"]

    await client.query(
        "RELATE $f->contains_chunk->$c",
        {"f": file_id, "c": chunk_id},
    )
    return chunk_id


async def relate_impl_to_type(
    client: AsyncSurreal,
    impl_id,
    type_id,
    trait_name: str | None = None,
):
    """Create graph edge: impl chunk -> implements -> struct/enum chunk."""
    if trait_name:
        await client.query(
            "RELATE $impl_id->implements->$type_id SET trait_name = $trait",
            {"impl_id": impl_id, "type_id": type_id, "trait": trait_name},
        )
    else:
        await client.query(
            "RELATE $impl_id->implements->$type_id",
            {"impl_id": impl_id, "type_id": type_id},
        )


def _flatten_results(result) -> list[dict]:
    """Flatten nested query results from the SurrealDB SDK."""
    if not result:
        return []
    if isinstance(result, list) and len(result) > 0 and isinstance(result[0], list):
        return result[-1]
    if isinstance(result, list):
        return [r for r in result if isinstance(r, dict)]
    return []


async def search_similar(
    client: AsyncSurreal, query_embedding: list[float], limit: int = 5
) -> list[dict]:
    """Perform vector similarity search on code chunks using cosine similarity."""
    result = await client.query(
        f"""
        SELECT
            id, kind, name, source_text, signature,
            vector::similarity::cosine(embedding, $embedding) AS score
        FROM chunk
        ORDER BY score DESC
        LIMIT {limit}
        """,
        {"embedding": query_embedding},
    )
    return _flatten_results(result)


async def search_with_graph(
    client: AsyncSurreal, query_embedding: list[float], limit: int = 5
) -> list[dict]:
    """Vector search + graph expansion for richer context."""
    # First: find similar chunks
    similar = await search_similar(client, query_embedding, limit)
    if not similar:
        return []

    # Deduplicate by chunk ID
    seen = set()
    unique = []
    for r in similar:
        rid = str(r.get("id", ""))
        if rid not in seen:
            seen.add(rid)
            unique.append(r)
    similar = unique

    # Second: enrich each chunk with graph context
    chunk_ids = [r["id"] for r in similar]
    raw_enriched = await client.query(
        """
        SELECT
            *,
            <-contains_chunk<-file.relative_path AS file_path,
            <-contains_chunk<-file<-contains_file<-codebase.name AS codebase_name,
            ->implements->chunk AS implements_types,
            <-implements<-chunk AS implemented_by
        FROM $ids
        """,
        {"ids": chunk_ids},
    )
    enriched = _flatten_results(raw_enriched)

    # Deduplicate enriched results by ID
    seen = set()
    unique_enriched = []
    for item in enriched:
        rid = str(item.get("id", ""))
        if rid not in seen:
            seen.add(rid)
            unique_enriched.append(item)

    # Merge scores back in
    score_map = {str(r["id"]): r["score"] for r in similar}
    for item in unique_enriched:
        item["score"] = score_map.get(str(item.get("id")), 0)
    unique_enriched.sort(key=lambda x: x.get("score", 0), reverse=True)

    return unique_enriched
