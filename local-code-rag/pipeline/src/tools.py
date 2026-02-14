"""Tool definitions for LLM tool calling in the RAG pipeline."""

import httpx


def search_web(query: str, max_results: int = 5) -> str:
    """Search the web for information about programming concepts, frameworks, etc.

    Args:
        query: The search query (e.g., "What is WASM WASI interface")
        max_results: Maximum number of results to return

    Returns:
        Formatted search results with titles, URLs, and snippets
    """
    try:
        response = httpx.get(
            "https://api.duckduckgo.com/",
            params={"q": query, "format": "json", "no_html": 1, "skip_disambig": 1},
            timeout=10.0,
        )
        response.raise_for_status()
        data = response.json()
    except Exception as e:
        return f"Web search failed: {e}"

    results = []

    # Abstract (main answer if available)
    if data.get("Abstract"):
        results.append(f"**Summary:** {data['Abstract']}")
        if data.get("AbstractURL"):
            results.append(f"Source: {data['AbstractURL']}")
        results.append("")

    # Related topics
    related = data.get("RelatedTopics", [])
    count = 0
    for topic in related:
        if count >= max_results:
            break

        # Direct topic with Text and FirstURL
        if isinstance(topic, dict) and topic.get("Text"):
            text = topic["Text"]
            url = topic.get("FirstURL", "")
            results.append(f"- {text}")
            if url:
                results.append(f"  URL: {url}")
            count += 1

        # Nested topics (subcategories)
        elif isinstance(topic, dict) and topic.get("Topics"):
            for subtopic in topic["Topics"]:
                if count >= max_results:
                    break
                if subtopic.get("Text"):
                    text = subtopic["Text"]
                    url = subtopic.get("FirstURL", "")
                    results.append(f"- {text}")
                    if url:
                        results.append(f"  URL: {url}")
                    count += 1

    # Definition (if no abstract)
    if not results and data.get("Definition"):
        results.append(f"**Definition:** {data['Definition']}")
        if data.get("DefinitionURL"):
            results.append(f"Source: {data['DefinitionURL']}")

    # Answer (for some queries)
    if not results and data.get("Answer"):
        results.append(f"**Answer:** {data['Answer']}")

    if not results:
        return f"No web results found for: {query}"

    return "\n".join(results)


# Tool schema for Ollama tool calling
SEARCH_WEB_TOOL = {
    "type": "function",
    "function": {
        "name": "search_web",
        "description": "Search the web for information about programming concepts, "
        "frameworks, standards, technologies, or any topic not found in the codebase. "
        "Use this when you need external knowledge to answer questions about WASM, WASI, "
        "HTTP standards, libraries, or other technical concepts.",
        "parameters": {
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to look up",
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default 5)",
                },
            },
            "required": ["query"],
        },
    },
}

# Map of tool names to functions
TOOL_FUNCTIONS = {
    "search_web": search_web,
}

# List of all available tools
AVAILABLE_TOOLS = [SEARCH_WEB_TOOL]
