"""Graph CRUD operations for the knowledge graph REPL."""

from surrealdb import AsyncSurreal


async def create_node(client: AsyncSurreal, table: str, data: dict) -> str:
    """Create a new node in the specified table.

    Args:
        client: SurrealDB client
        table: Table name (e.g., 'chunk', 'file', 'codebase')
        data: Node data (key-value pairs)

    Returns:
        The ID of the created node
    """
    result = await client.create(table, data)
    return str(result.get("id", ""))


async def get_node(client: AsyncSurreal, node_id: str) -> dict | None:
    """Get a node by its ID.

    Args:
        client: SurrealDB client
        node_id: Full node ID (e.g., 'chunk:abc123')

    Returns:
        Node data dict or None if not found
    """
    result = await client.query(
        "SELECT * FROM $id",
        {"id": node_id},
    )
    if result and isinstance(result[0], list) and result[0]:
        return result[0][0]
    if result and isinstance(result[0], dict):
        return result[0]
    return None


async def update_node(client: AsyncSurreal, node_id: str, data: dict) -> dict | None:
    """Update a node by its ID.

    Args:
        client: SurrealDB client
        node_id: Full node ID (e.g., 'chunk:abc123')
        data: Fields to update

    Returns:
        Updated node data or None if not found
    """
    # Build SET clause dynamically
    set_parts = []
    params = {"id": node_id}
    for i, (key, value) in enumerate(data.items()):
        param_name = f"v{i}"
        set_parts.append(f"{key} = ${param_name}")
        params[param_name] = value

    if not set_parts:
        return await get_node(client, node_id)

    query = f"UPDATE $id SET {', '.join(set_parts)} RETURN AFTER"
    result = await client.query(query, params)

    if result and isinstance(result[0], list) and result[0]:
        return result[0][0]
    return None


async def delete_node(client: AsyncSurreal, node_id: str) -> bool:
    """Delete a node by its ID.

    Args:
        client: SurrealDB client
        node_id: Full node ID (e.g., 'chunk:abc123')

    Returns:
        True if deleted, False otherwise
    """
    result = await client.query(
        "DELETE $id RETURN BEFORE",
        {"id": node_id},
    )
    return bool(result and result[0])


async def list_nodes(
    client: AsyncSurreal, table: str, limit: int = 10, offset: int = 0
) -> list[dict]:
    """List nodes from a table.

    Args:
        client: SurrealDB client
        table: Table name
        limit: Maximum number of results
        offset: Number of results to skip

    Returns:
        List of node dicts
    """
    result = await client.query(
        f"SELECT * FROM {table} LIMIT $limit START $offset",
        {"limit": limit, "offset": offset},
    )
    if result and isinstance(result[0], list):
        return result[0]
    return []


async def create_edge(
    client: AsyncSurreal,
    from_id: str,
    relation: str,
    to_id: str,
    data: dict | None = None,
) -> str:
    """Create an edge between two nodes.

    Args:
        client: SurrealDB client
        from_id: Source node ID
        relation: Edge type (e.g., 'implements', 'contains_file')
        to_id: Target node ID
        data: Optional edge data

    Returns:
        The ID of the created edge
    """
    if data:
        set_parts = []
        params = {"from": from_id, "to": to_id}
        for i, (key, value) in enumerate(data.items()):
            param_name = f"v{i}"
            set_parts.append(f"{key} = ${param_name}")
            params[param_name] = value
        query = f"RELATE $from->{relation}->$to SET {', '.join(set_parts)}"
    else:
        query = f"RELATE $from->{relation}->$to"
        params = {"from": from_id, "to": to_id}

    result = await client.query(query, params)

    if result and isinstance(result[0], list) and result[0]:
        return str(result[0][0].get("id", ""))
    return ""


async def delete_edge(client: AsyncSurreal, edge_id: str) -> bool:
    """Delete an edge by its ID.

    Args:
        client: SurrealDB client
        edge_id: Full edge ID (e.g., 'implements:xyz789')

    Returns:
        True if deleted, False otherwise
    """
    result = await client.query(
        "DELETE $id RETURN BEFORE",
        {"id": edge_id},
    )
    return bool(result and result[0])


async def traverse(
    client: AsyncSurreal,
    start_id: str,
    direction: str = "out",
    depth: int = 1,
) -> list[dict]:
    """Traverse the graph from a starting node.

    Args:
        client: SurrealDB client
        start_id: Starting node ID
        direction: 'out' for outgoing edges, 'in' for incoming, 'both' for both
        depth: How many hops to traverse

    Returns:
        List of traversal results with edge and node info
    """
    if direction == "out":
        arrow = "->"
    elif direction == "in":
        arrow = "<-"
    else:
        arrow = "<->"

    # Build traversal query based on depth
    # For depth 1: SELECT *, ->edge->* as targets FROM $id
    # For deeper, we use recursive traversal
    if depth == 1:
        if direction == "out":
            query = """
                SELECT
                    id,
                    kind,
                    name,
                    ->implements->chunk AS implements,
                    ->contains_file->file AS contains_files,
                    ->contains_chunk->chunk AS contains_chunks
                FROM $id
            """
        elif direction == "in":
            query = """
                SELECT
                    id,
                    kind,
                    name,
                    <-implements<-chunk AS implemented_by,
                    <-contains_file<-codebase AS in_codebase,
                    <-contains_chunk<-file AS in_file
                FROM $id
            """
        else:
            query = """
                SELECT
                    id,
                    kind,
                    name,
                    ->implements->chunk AS implements,
                    <-implements<-chunk AS implemented_by,
                    ->contains_file->file AS contains_files,
                    <-contains_file<-codebase AS in_codebase,
                    ->contains_chunk->chunk AS contains_chunks,
                    <-contains_chunk<-file AS in_file
                FROM $id
            """
    else:
        # For deeper traversal, use graph traversal syntax
        query = f"""
            SELECT
                id,
                kind,
                name,
                {arrow}(implements, contains_file, contains_chunk){{{depth}}} AS related
            FROM $id
        """

    result = await client.query(query, {"id": start_id})

    if result and isinstance(result[0], list):
        return result[0]
    if result and isinstance(result[0], dict):
        return [result[0]]
    return []


async def raw_query(client: AsyncSurreal, surql: str) -> list:
    """Execute a raw SurrealQL query.

    Args:
        client: SurrealDB client
        surql: Raw SurrealQL query string

    Returns:
        Query results
    """
    result = await client.query(surql)
    return result


async def search_similar(
    client: AsyncSurreal, query_embedding: list[float], limit: int = 5
) -> list[dict]:
    """Search for similar chunks using vector similarity.

    Args:
        client: SurrealDB client
        query_embedding: Query embedding vector
        limit: Maximum number of results

    Returns:
        List of similar chunks with scores
    """
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

    if result and isinstance(result[0], list):
        return result[0]
    return []


async def get_stats(client: AsyncSurreal) -> dict:
    """Get database statistics.

    Args:
        client: SurrealDB client

    Returns:
        Dict with counts for each table type
    """
    result = await client.query(
        """
        RETURN {
            codebases: (SELECT count() FROM codebase GROUP ALL)[0].count OR 0,
            files: (SELECT count() FROM file GROUP ALL)[0].count OR 0,
            chunks: (SELECT count() FROM chunk GROUP ALL)[0].count OR 0,
            concepts: (SELECT count() FROM concept GROUP ALL)[0].count OR 0,
            contains_file_edges: (SELECT count() FROM contains_file GROUP ALL)[0].count OR 0,
            contains_chunk_edges: (SELECT count() FROM contains_chunk GROUP ALL)[0].count OR 0,
            implements_edges: (SELECT count() FROM implements GROUP ALL)[0].count OR 0,
            relates_to_edges: (SELECT count() FROM relates_to GROUP ALL)[0].count OR 0
        }
        """
    )

    if result and result[0]:
        return result[0]
    return {}
