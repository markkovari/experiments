"""RAG module: retrieval-augmented generation over ingested codebases."""

import ollama

from .embedder import embed_text
from .store import connect, search_with_graph

DEFAULT_MODEL = "qwen2.5-coder:7b"

SYSTEM_PROMPT = """\
You are a code assistant that answers questions about codebases.
You will be given relevant code snippets retrieved from a vector+graph database.
Use ONLY the provided context to answer. If the context doesn't contain enough
information, say so. Reference specific functions, structs, and file paths in
your answers."""

CONTEXT_TEMPLATE = """\
## {kind}: `{name}`
**Codebase:** {codebase} | **File:** {file_path}
**Score:** {score:.4f}

```rust
{source_text}
```
"""


def format_context(results: list[dict]) -> str:
    """Format search results into a context block for the LLM."""
    sections = []
    for r in results:

        codebase = r.get("codebase_name", ["unknown"])
        if isinstance(codebase, list):
            codebase = codebase[0] if codebase else "unknown"

        file_path = r.get("file_path", ["unknown"])
        if isinstance(file_path, list):
            file_path = file_path[0] if file_path else "unknown"

        sections.append(
            CONTEXT_TEMPLATE.format(
                kind=r.get("kind", "unknown"),
                name=r.get("name", "unknown"),
                codebase=codebase,
                file_path=file_path,
                score=r.get("score", 0),
                source_text=r.get("source_text", ""),
            )
        )

    return "\n---\n".join(sections)


async def query_rag(
    question: str,
    ollama_host: str = "http://localhost:11434",
    surreal_url: str = "ws://localhost:8000/rpc",
    model: str = DEFAULT_MODEL,
    limit: int = 5,
    show_context: bool = False,
) -> str:
    """Full RAG pipeline: embed question -> search -> LLM answer."""

    # 1. Embed the question
    query_embedding = embed_text(question, host=ollama_host)

    # 2. Retrieve relevant code chunks with graph context
    client = await connect(url=surreal_url)
    results = await search_with_graph(client, query_embedding, limit=limit)

    if not results:
        return "No relevant code found in the database. Have you ingested any codebases?"

    # 3. Format context
    context = format_context(results)

    if show_context:
        print("\n=== Retrieved Context ===")
        print(context)
        print("=========================\n")

    # 4. Build the prompt and call the LLM
    user_prompt = f"""Here are relevant code snippets from the codebase:

{context}

---

**Question:** {question}

Please answer based on the code above."""

    llm_client = ollama.Client(host=ollama_host)
    response = llm_client.chat(
        model=model,
        messages=[
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt},
        ],
    )

    return response["message"]["content"]
