from surrealdb import Surreal

from .models import CodeChunk, FileInfo, CodebaseInfo

SURREAL_URL = "ws://localhost:8000/rpc"
SURREAL_NS = "code_rag"
SURREAL_DB = "code_rag"


async def connect(
    url: str = SURREAL_URL, ns: str = SURREAL_NS, db: str = SURREAL_DB
) -> Surreal:
    client = Surreal(url)
    await client.signin({"user": "root", "pass": "root"})
    await client.use(ns, db)
    return client


async def store_codebase(client: Surreal, codebase: CodebaseInfo) -> str:
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


async def store_file(client: Surreal, file_info: FileInfo, codebase_id: str) -> str:
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
        "RELATE $codebase->contains_file->$file",
        {"codebase": codebase_id, "file": file_id},
    )
    return file_id


async def store_chunk(client: Surreal, chunk: CodeChunk, file_id: str) -> str:
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
        "RELATE $file->contains_chunk->$chunk",
        {"file": file_id, "chunk": chunk_id},
    )
    return chunk_id


async def relate_impl_to_type(
    client: Surreal,
    impl_id: str,
    type_id: str,
    trait_name: str | None = None,
):
    """Create graph edge: impl chunk -> implements -> struct/enum chunk."""
    params = {"impl_id": impl_id, "type_id": type_id}
    if trait_name:
        await client.query(
            "RELATE $impl_id->implements->$type_id SET trait_name = $trait",
            {**params, "trait": trait_name},
        )
    else:
        await client.query("RELATE $impl_id->implements->$type_id", params)


async def search_similar(
    client: Surreal, query_embedding: list[float], limit: int = 5
) -> list[dict]:
    """Perform vector similarity search on code chunks."""
    result = await client.query(
        """
        SELECT
            id, kind, name, source_text, signature,
            vector::distance::knn() AS distance
        FROM chunk
        WHERE embedding <|$limit|> $embedding
        ORDER BY distance
        """,
        {"embedding": query_embedding, "limit": limit},
    )
    return result


async def search_with_graph(
    client: Surreal, query_embedding: list[float], limit: int = 5
) -> list[dict]:
    """Vector search + graph expansion for richer context."""
    result = await client.query(
        """
        LET $similar = (
            SELECT
                id, kind, name, source_text, signature,
                vector::distance::knn() AS distance
            FROM chunk
            WHERE embedding <|$limit|> $embedding
            ORDER BY distance
        );

        SELECT
            *,
            <-contains_chunk<-file.relative_path AS file_path,
            <-contains_chunk<-file<-contains_file<-codebase.name AS codebase_name,
            ->implements->chunk AS implements_types,
            <-implements<-chunk AS implemented_by
        FROM $similar;
        """,
        {"embedding": query_embedding, "limit": limit},
    )
    return result
