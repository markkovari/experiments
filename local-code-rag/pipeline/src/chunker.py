"""Orchestrates the full pipeline: parse -> embed -> store."""

import os
from pathlib import Path

from .models import CodebaseInfo, FileInfo
from .parser import parse_file as parse_rust_file
from .config_parser import parse_config_file
from .embedder import embed_all
from .store import connect, store_codebase, store_file, store_chunk, relate_impl_to_type

# File extensions that tree-sitter handles (AST parsing)
TREESITTER_EXTENSIONS = {".rs"}

# Config/descriptor files parsed by section splitting
CONFIG_EXTENSIONS = {".toml", ".yaml", ".yml", ".wit"}

# All extensions we care about
ALL_EXTENSIONS = TREESITTER_EXTENSIONS | CONFIG_EXTENSIONS


def _detect_language(path: Path) -> str:
    suffix = path.suffix
    if suffix == ".rs":
        return "rust"
    if suffix == ".toml":
        return "toml"
    if suffix in (".yaml", ".yml"):
        return "yaml"
    if suffix == ".wit":
        return "wit"
    if suffix == ".zig":
        return "zig"
    return "unknown"


def _collect_files(codebase_path: Path) -> list[Path]:
    """Collect all ingestable files from a codebase, skipping build artifacts."""
    skip_dirs = {"target", "build", ".git", "node_modules", "__pycache__"}
    files = []
    for f in sorted(codebase_path.rglob("*")):
        if not f.is_file():
            continue
        if any(part in skip_dirs for part in f.parts):
            continue
        if f.suffix in ALL_EXTENSIONS:
            files.append(f)
    return files


async def ingest_codebase(
    codebase_path: str,
    codebase_name: str,
    ollama_host: str = "http://localhost:11434",
    surreal_url: str = "ws://localhost:8000/rpc",
):
    """Ingest a codebase: parse source + config files, embed with Ollama, store in SurrealDB."""
    codebase_path = os.path.abspath(codebase_path)
    base = Path(codebase_path)
    print(f"Ingesting codebase '{codebase_name}' from {codebase_path}")

    # 1. Collect all files
    all_files = _collect_files(base)
    rs_count = sum(1 for f in all_files if f.suffix == ".rs")
    config_count = sum(1 for f in all_files if f.suffix in CONFIG_EXTENSIONS)
    print(f"  Found {len(all_files)} files ({rs_count} source, {config_count} config/descriptor)")

    if not all_files:
        print("  No files found, nothing to ingest.")
        return

    # 2. Parse each file
    all_chunks = []
    file_infos = []

    for filepath in all_files:
        source_bytes = filepath.read_bytes()
        source_text = source_bytes.decode("utf-8", errors="replace")
        relative = str(filepath.relative_to(base))
        language = _detect_language(filepath)

        file_info = FileInfo(
            path=str(filepath),
            relative_path=relative,
            language=language,
            size_bytes=len(source_bytes),
        )

        if filepath.suffix in TREESITTER_EXTENSIONS:
            chunks = parse_rust_file(source_bytes, str(filepath), relative)
        elif filepath.suffix in CONFIG_EXTENSIONS:
            _, chunks = parse_config_file(source_text, str(filepath), relative)
        else:
            chunks = []

        file_info.chunks = chunks
        file_infos.append(file_info)
        all_chunks.extend(chunks)
        print(f"  Parsed {relative} ({language}): {len(chunks)} chunks")

    print(f"  Total chunks: {len(all_chunks)}")

    # 3. Embed all chunks
    print("  Embedding chunks with Ollama...")
    texts = [_embedding_text(c) for c in all_chunks]
    embeddings = embed_all(texts, host=ollama_host)
    for i, emb in enumerate(embeddings):
        all_chunks[i].embedding = emb

    # 4. Store in SurrealDB
    print("  Storing in SurrealDB...")
    client = await connect(url=surreal_url)

    codebase_info = CodebaseInfo(
        name=codebase_name,
        path=codebase_path,
        language="multi",
        files=file_infos,
    )
    codebase_id = await store_codebase(client, codebase_info)
    print(f"  Created codebase node: {codebase_id}")

    # Track chunk IDs for relationship building
    chunk_id_map = {}  # (relative_path, name, kind) -> db_id

    for file_info in file_infos:
        file_id = await store_file(client, file_info, codebase_id)

        for chunk in file_info.chunks:
            chunk_db_id = await store_chunk(client, chunk, file_id)
            chunk_id_map[(chunk.relative_path, chunk.name, chunk.kind)] = chunk_db_id

    # 5. Build graph relationships
    print("  Building graph relationships...")
    await _build_relationships(chunk_id_map, file_infos, client)

    print(f"  Done! Ingested {len(all_chunks)} chunks from {len(file_infos)} files.")


def _embedding_text(chunk) -> str:
    """Format a chunk for embedding, using language-aware context."""
    fmt = chunk.metadata.get("format", "")
    if fmt == "toml":
        return f"TOML configuration '{chunk.name}' in {chunk.relative_path}:\n{chunk.source_text}"
    if fmt == "yaml":
        return f"YAML manifest '{chunk.name}' in {chunk.relative_path}:\n{chunk.source_text}"
    if fmt == "wit":
        return f"WIT interface definition '{chunk.name}' in {chunk.relative_path}:\n{chunk.source_text}"
    return f"Rust {chunk.kind} '{chunk.name}' in {chunk.relative_path}:\n{chunk.source_text}"


async def _build_relationships(
    chunk_id_map: dict, file_infos: list[FileInfo], client
):
    """Build impl->struct/enum graph edges."""
    type_chunks = {}  # name -> (relative_path, name, kind)
    impl_chunks = []  # list of (impl_key, type_name, trait_name)

    for file_info in file_infos:
        for chunk in file_info.chunks:
            if chunk.kind in ("struct", "enum", "trait"):
                type_chunks[chunk.name] = (
                    chunk.relative_path,
                    chunk.name,
                    chunk.kind,
                )
            elif chunk.kind == "impl":
                trait_name = chunk.metadata.get("trait")
                impl_chunks.append(
                    (
                        (chunk.relative_path, chunk.name, chunk.kind),
                        chunk.name,
                        trait_name,
                    )
                )

    for impl_key, type_name, trait_name in impl_chunks:
        if impl_key in chunk_id_map and type_name in type_chunks:
            type_key = type_chunks[type_name]
            if type_key in chunk_id_map:
                await relate_impl_to_type(
                    client,
                    chunk_id_map[impl_key],
                    chunk_id_map[type_key],
                    trait_name,
                )
                print(
                    f"    RELATE impl {type_name}"
                    + (f" (trait: {trait_name})" if trait_name else "")
                    + f" -> {type_name}"
                )
