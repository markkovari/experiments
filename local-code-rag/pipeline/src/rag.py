"""RAG module: retrieval-augmented generation over ingested codebases."""

import json

import ollama

from .embedder import embed_text
from .store import connect, search_with_graph
from .tools import AVAILABLE_TOOLS, TOOL_FUNCTIONS

DEFAULT_MODEL = "qwen2.5-coder:7b"

SYSTEM_PROMPT = """\
You are a code assistant with access to:
1. Retrieved code snippets from a local codebase
2. Web search for external knowledge (standards, frameworks, concepts)

Use the provided codebase context first. When you need external information about
technologies, standards, or concepts not in the codebase (like WASM, WASI, HTTP
specifications, library documentation), use the search_web tool.

When referencing code, mention specific functions, structs, and file paths.
If the context doesn't contain enough information and web search doesn't help,
say so clearly."""

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


def execute_tool_call(tool_call: dict) -> str:
    """Execute a tool call and return the result."""
    name = tool_call.get("function", {}).get("name", "")
    args = tool_call.get("function", {}).get("arguments", {})

    # Arguments may be a string (JSON) or dict
    if isinstance(args, str):
        try:
            args = json.loads(args)
        except json.JSONDecodeError:
            args = {}

    if name in TOOL_FUNCTIONS:
        try:
            result = TOOL_FUNCTIONS[name](**args)
            return result
        except Exception as e:
            return f"Tool execution error: {e}"
    else:
        return f"Unknown tool: {name}"


async def query_rag(
    question: str,
    ollama_host: str = "http://localhost:11434",
    surreal_url: str = "ws://localhost:8000/rpc",
    model: str = DEFAULT_MODEL,
    limit: int = 5,
    show_context: bool = False,
    max_tool_calls: int = 3,
) -> str:
    """Full RAG pipeline: embed question -> search -> LLM answer with tool calling."""

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

    # 4. Build the initial prompt
    user_prompt = f"""Here are relevant code snippets from the codebase:

{context}

---

**Question:** {question}

Please answer based on the code above. If you need additional information about external
technologies, standards, or concepts (like WASM, WASI, HTTP specs), use the search_web tool."""

    # 5. Build messages for LLM
    messages = [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": user_prompt},
    ]

    llm_client = ollama.Client(host=ollama_host)

    # 6. Tool calling loop
    for _ in range(max_tool_calls):
        response = llm_client.chat(
            model=model,
            messages=messages,
            tools=AVAILABLE_TOOLS,
        )

        message = response.get("message", {})

        # Check if the model wants to call tools
        tool_calls = message.get("tool_calls", [])

        if not tool_calls:
            # No tool calls, return the final answer
            return message.get("content", "")

        # Execute each tool call
        print(f"[RAG] Executing {len(tool_calls)} tool call(s)...")
        messages.append(message)  # Add assistant message with tool_calls

        for tool_call in tool_calls:
            tool_name = tool_call.get("function", {}).get("name", "unknown")
            print(f"[RAG] Calling tool: {tool_name}")

            result = execute_tool_call(tool_call)

            # Add tool result to messages
            messages.append({"role": "tool", "content": result})

    # If we exhausted tool calls, make one final call without tools
    response = llm_client.chat(
        model=model,
        messages=messages,
    )

    return response.get("message", {}).get("content", "")
