"""Interactive REPL for knowledge graph exploration and modification."""

import asyncio
import cmd
import shlex
from typing import Any

from .embedder import embed_text
from .graph_ops import (
    create_edge,
    create_node,
    delete_edge,
    delete_node,
    get_node,
    get_stats,
    list_nodes,
    raw_query,
    search_similar,
    traverse,
    update_node,
)
from .store import connect
from .tools import search_web


class GraphREPL(cmd.Cmd):
    """Interactive REPL for exploring and modifying the knowledge graph."""

    intro = """
╔══════════════════════════════════════════════════════════════════╗
║           Knowledge Graph REPL - Code RAG Pipeline               ║
╠══════════════════════════════════════════════════════════════════╣
║  Commands: node, edge, query, traverse, search, embed, web, stats║
║  Type 'help <command>' for detailed usage                        ║
║  Type 'exit' or 'quit' to leave                                  ║
╚══════════════════════════════════════════════════════════════════╝
"""
    prompt = "graph> "

    def __init__(
        self,
        surreal_url: str = "ws://localhost:8000/rpc",
        ollama_host: str = "http://localhost:11434",
    ):
        super().__init__()
        self.surreal_url = surreal_url
        self.ollama_host = ollama_host
        self._client = None
        self._loop = asyncio.new_event_loop()

    def _run_async(self, coro) -> Any:
        """Run an async coroutine in the event loop."""
        return self._loop.run_until_complete(coro)

    def _get_client(self):
        """Get or create the SurrealDB client."""
        if self._client is None:
            self._client = self._run_async(connect(url=self.surreal_url))
        return self._client

    def _parse_kv_args(self, args: list[str]) -> dict:
        """Parse --key=value or --key value arguments into a dict."""
        result = {}
        i = 0
        while i < len(args):
            arg = args[i]
            if arg.startswith("--"):
                key = arg[2:]
                if "=" in key:
                    key, value = key.split("=", 1)
                    result[key] = value
                elif i + 1 < len(args) and not args[i + 1].startswith("--"):
                    result[key] = args[i + 1]
                    i += 1
                else:
                    result[key] = True
            i += 1
        return result

    # ==================== NODE COMMANDS ====================

    def do_node(self, arg: str):
        """Node operations: create, get, update, delete, list.

        Usage:
            node create <table> --name <name> --kind <kind> [--key value ...]
            node get <id>
            node update <id> --key value [--key value ...]
            node delete <id>
            node list <table> [--limit N] [--offset N]

        Examples:
            node create chunk --name my_func --kind function --source_text "fn foo() {}"
            node get chunk:abc123
            node update chunk:abc123 --name new_name
            node delete chunk:abc123
            node list chunk --limit 10
        """
        try:
            parts = shlex.split(arg)
        except ValueError as e:
            print(f"Parse error: {e}")
            return

        if not parts:
            print("Usage: node <create|get|update|delete|list> ...")
            return

        subcommand = parts[0]
        args = parts[1:]

        if subcommand == "create":
            self._node_create(args)
        elif subcommand == "get":
            self._node_get(args)
        elif subcommand == "update":
            self._node_update(args)
        elif subcommand == "delete":
            self._node_delete(args)
        elif subcommand == "list":
            self._node_list(args)
        else:
            print(f"Unknown node subcommand: {subcommand}")
            print("Available: create, get, update, delete, list")

    def _node_create(self, args: list[str]):
        if not args:
            print("Usage: node create <table> --key value ...")
            return

        table = args[0]
        data = self._parse_kv_args(args[1:])

        if not data:
            print("No data provided. Use --key value pairs.")
            return

        try:
            client = self._get_client()
            node_id = self._run_async(create_node(client, table, data))
            print(f"Created: {node_id}")
        except Exception as e:
            print(f"Error creating node: {e}")

    def _node_get(self, args: list[str]):
        if not args:
            print("Usage: node get <id>")
            return

        node_id = args[0]

        try:
            client = self._get_client()
            node = self._run_async(get_node(client, node_id))
            if node:
                self._print_node(node)
            else:
                print(f"Node not found: {node_id}")
        except Exception as e:
            print(f"Error getting node: {e}")

    def _node_update(self, args: list[str]):
        if not args:
            print("Usage: node update <id> --key value ...")
            return

        node_id = args[0]
        data = self._parse_kv_args(args[1:])

        if not data:
            print("No updates provided. Use --key value pairs.")
            return

        try:
            client = self._get_client()
            node = self._run_async(update_node(client, node_id, data))
            if node:
                print(f"Updated: {node_id}")
                self._print_node(node)
            else:
                print(f"Node not found: {node_id}")
        except Exception as e:
            print(f"Error updating node: {e}")

    def _node_delete(self, args: list[str]):
        if not args:
            print("Usage: node delete <id>")
            return

        node_id = args[0]

        try:
            client = self._get_client()
            deleted = self._run_async(delete_node(client, node_id))
            if deleted:
                print(f"Deleted: {node_id}")
            else:
                print(f"Node not found or already deleted: {node_id}")
        except Exception as e:
            print(f"Error deleting node: {e}")

    def _node_list(self, args: list[str]):
        if not args:
            print("Usage: node list <table> [--limit N] [--offset N]")
            return

        table = args[0]
        opts = self._parse_kv_args(args[1:])
        limit = int(opts.get("limit", 10))
        offset = int(opts.get("offset", 0))

        try:
            client = self._get_client()
            nodes = self._run_async(list_nodes(client, table, limit, offset))

            if not nodes:
                print(f"No nodes found in table '{table}'")
                return

            print(f"\n{table} nodes (showing {len(nodes)}):")
            print("-" * 60)
            for node in nodes:
                node_id = node.get("id", "?")
                kind = node.get("kind", "")
                name = node.get("name", "")
                if kind and name:
                    print(f"  [{node_id}] {kind}: {name}")
                elif name:
                    print(f"  [{node_id}] {name}")
                else:
                    print(f"  [{node_id}]")
        except Exception as e:
            print(f"Error listing nodes: {e}")

    def _print_node(self, node: dict):
        """Pretty print a node."""
        print()
        for key, value in node.items():
            if key == "embedding":
                print(f"  {key}: [{len(value)} floats]")
            elif key == "source_text" and isinstance(value, str) and len(value) > 100:
                print(f"  {key}: {value[:100]}...")
            else:
                print(f"  {key}: {value}")
        print()

    # ==================== EDGE COMMANDS ====================

    def do_edge(self, arg: str):
        """Edge operations: create, delete.

        Usage:
            edge create <from_id> <relation> <to_id> [--key value ...]
            edge delete <edge_id>

        Examples:
            edge create chunk:abc123 implements chunk:def456 --trait Display
            edge delete implements:xyz789
        """
        try:
            parts = shlex.split(arg)
        except ValueError as e:
            print(f"Parse error: {e}")
            return

        if not parts:
            print("Usage: edge <create|delete> ...")
            return

        subcommand = parts[0]
        args = parts[1:]

        if subcommand == "create":
            self._edge_create(args)
        elif subcommand == "delete":
            self._edge_delete(args)
        else:
            print(f"Unknown edge subcommand: {subcommand}")
            print("Available: create, delete")

    def _edge_create(self, args: list[str]):
        if len(args) < 3:
            print("Usage: edge create <from_id> <relation> <to_id> [--key value ...]")
            return

        from_id = args[0]
        relation = args[1]
        to_id = args[2]
        data = self._parse_kv_args(args[3:]) if len(args) > 3 else None

        try:
            client = self._get_client()
            edge_id = self._run_async(
                create_edge(client, from_id, relation, to_id, data)
            )
            if edge_id:
                print(f"Created: {edge_id}")
            else:
                print("Edge created (no ID returned)")
        except Exception as e:
            print(f"Error creating edge: {e}")

    def _edge_delete(self, args: list[str]):
        if not args:
            print("Usage: edge delete <edge_id>")
            return

        edge_id = args[0]

        try:
            client = self._get_client()
            deleted = self._run_async(delete_edge(client, edge_id))
            if deleted:
                print(f"Deleted: {edge_id}")
            else:
                print(f"Edge not found or already deleted: {edge_id}")
        except Exception as e:
            print(f"Error deleting edge: {e}")

    # ==================== QUERY COMMAND ====================

    def do_query(self, arg: str):
        """Execute a raw SurrealQL query.

        Usage:
            query <surql>

        Examples:
            query SELECT * FROM chunk WHERE kind = 'function'
            query SELECT count() FROM chunk GROUP ALL
            query SELECT * FROM chunk LIMIT 5
        """
        if not arg.strip():
            print("Usage: query <surql>")
            return

        try:
            client = self._get_client()
            result = self._run_async(raw_query(client, arg))

            if not result:
                print("No results")
                return

            # Pretty print results
            for i, item in enumerate(result):
                if isinstance(item, list):
                    print(f"\nResult set {i + 1} ({len(item)} rows):")
                    for row in item[:20]:  # Limit display
                        self._print_result_row(row)
                    if len(item) > 20:
                        print(f"  ... and {len(item) - 20} more rows")
                else:
                    print(f"\nResult {i + 1}:")
                    self._print_result_row(item)
        except Exception as e:
            print(f"Query error: {e}")

    def _print_result_row(self, row):
        """Print a single result row."""
        if isinstance(row, dict):
            # Compact dict display
            parts = []
            for k, v in row.items():
                if k == "embedding":
                    parts.append(f"{k}:[{len(v)} floats]")
                elif k == "source_text" and isinstance(v, str) and len(v) > 50:
                    parts.append(f"{k}:{v[:50]}...")
                else:
                    parts.append(f"{k}:{v}")
            print(f"  {', '.join(parts)}")
        else:
            print(f"  {row}")

    # ==================== TRAVERSE COMMAND ====================

    def do_traverse(self, arg: str):
        """Traverse the graph from a starting node.

        Usage:
            traverse <node_id> [--direction out|in|both] [--depth N]

        Examples:
            traverse chunk:abc123
            traverse chunk:abc123 --direction out --depth 2
            traverse file:xyz --direction in
        """
        try:
            parts = shlex.split(arg)
        except ValueError as e:
            print(f"Parse error: {e}")
            return

        if not parts:
            print("Usage: traverse <node_id> [--direction out|in|both] [--depth N]")
            return

        node_id = parts[0]
        opts = self._parse_kv_args(parts[1:])
        direction = opts.get("direction", "out")
        depth = int(opts.get("depth", 1))

        try:
            client = self._get_client()
            results = self._run_async(traverse(client, node_id, direction, depth))

            if not results:
                print(f"No results for {node_id}")
                return

            print(f"\nTraversal from {node_id} ({direction}, depth={depth}):")
            print("-" * 60)
            for item in results:
                self._print_traversal_result(item)
        except Exception as e:
            print(f"Traversal error: {e}")

    def _print_traversal_result(self, item: dict):
        """Print a traversal result."""
        node_id = item.get("id", "?")
        kind = item.get("kind", "")
        name = item.get("name", "")

        print(f"\n[{node_id}] {kind}: {name}" if kind else f"\n[{node_id}] {name}")

        # Print relationships
        for key, value in item.items():
            if key in ("id", "kind", "name", "embedding", "source_text", "metadata"):
                continue
            if value and isinstance(value, list) and value:
                print(f"  {key}:")
                for v in value[:5]:
                    if isinstance(v, dict):
                        vid = v.get("id", v)
                        vname = v.get("name", "")
                        print(f"    -> {vid} ({vname})" if vname else f"    -> {vid}")
                    else:
                        print(f"    -> {v}")
                if len(value) > 5:
                    print(f"    ... and {len(value) - 5} more")

    # ==================== SEARCH COMMAND ====================

    def do_search(self, arg: str):
        """Search for similar code chunks using semantic search.

        Usage:
            search <query text> [--limit N]

        Examples:
            search HTTP handler
            search "struct with counter field" --limit 10
        """
        try:
            parts = shlex.split(arg)
        except ValueError as e:
            print(f"Parse error: {e}")
            return

        if not parts:
            print("Usage: search <query text> [--limit N]")
            return

        # Find --limit flag
        limit = 5
        query_parts = []
        i = 0
        while i < len(parts):
            if parts[i] == "--limit" and i + 1 < len(parts):
                limit = int(parts[i + 1])
                i += 2
            else:
                query_parts.append(parts[i])
                i += 1

        query = " ".join(query_parts)

        if not query:
            print("No search query provided")
            return

        try:
            print(f"Embedding query: '{query}'...")
            embedding = embed_text(query, host=self.ollama_host)

            client = self._get_client()
            results = self._run_async(search_similar(client, embedding, limit))

            if not results:
                print("No similar chunks found")
                return

            print(f"\nSearch results for '{query}' ({len(results)} matches):")
            print("-" * 60)
            for i, r in enumerate(results, 1):
                score = r.get("score", 0)
                kind = r.get("kind", "?")
                name = r.get("name", "?")
                source = r.get("source_text", "")

                print(f"\n{i}. [{kind}] {name} (score: {score:.4f})")
                # Show first few lines of source
                lines = source.split("\n")[:5]
                for line in lines:
                    print(f"   {line}")
                if len(source.split("\n")) > 5:
                    print(f"   ... ({len(source.split(chr(10))) - 5} more lines)")
        except Exception as e:
            print(f"Search error: {e}")

    # ==================== EMBED COMMAND ====================

    def do_embed(self, arg: str):
        """Generate an embedding for the given text.

        Usage:
            embed <text>

        Examples:
            embed HTTP request handler
            embed "function that parses JSON"
        """
        if not arg.strip():
            print("Usage: embed <text>")
            return

        try:
            print(f"Embedding: '{arg}'...")
            embedding = embed_text(arg, host=self.ollama_host)
            print(f"\nEmbedding ({len(embedding)} dimensions):")
            # Show first and last few values
            print(f"  [{embedding[0]:.6f}, {embedding[1]:.6f}, {embedding[2]:.6f}, ...")
            print(
                f"   ..., {embedding[-3]:.6f}, {embedding[-2]:.6f}, {embedding[-1]:.6f}]"
            )
        except Exception as e:
            print(f"Embedding error: {e}")

    # ==================== WEB COMMAND ====================

    def do_web(self, arg: str):
        """Web search and knowledge graph enrichment.

        Usage:
            web search <query>                    Search web and display results
            web add <query> [--link-to <id>]      Search, embed, and store as concept node
            web list [--limit N]                  List stored concept nodes

        Examples:
            web search "WASM WASI interface"
            web add "WebAssembly System Interface" --link-to chunk:abc123
            web add "Rust HTTP server patterns"
            web list --limit 10
        """
        try:
            parts = shlex.split(arg)
        except ValueError as e:
            print(f"Parse error: {e}")
            return

        if not parts:
            print("Usage: web <search|add|list> ...")
            return

        subcommand = parts[0]
        args = parts[1:]

        if subcommand == "search":
            self._web_search(args)
        elif subcommand == "add":
            self._web_add(args)
        elif subcommand == "list":
            self._web_list(args)
        else:
            print(f"Unknown web subcommand: {subcommand}")
            print("Available: search, add, list")

    def _web_search(self, args: list[str]):
        """Search the web and display results."""
        if not args:
            print("Usage: web search <query>")
            return

        query = " ".join(args)
        print(f"Searching web for: '{query}'...")

        try:
            results = search_web(query, max_results=5)
            print(f"\n{results}")
        except Exception as e:
            print(f"Web search error: {e}")

    def _web_add(self, args: list[str]):
        """Search web, embed results, and store as concept node."""
        if not args:
            print("Usage: web add <query> [--link-to <chunk_id>]")
            return

        # Parse --link-to flag
        link_to = None
        query_parts = []
        i = 0
        while i < len(args):
            if args[i] == "--link-to" and i + 1 < len(args):
                link_to = args[i + 1]
                i += 2
            else:
                query_parts.append(args[i])
                i += 1

        query = " ".join(query_parts)
        if not query:
            print("No query provided")
            return

        print(f"Searching web for: '{query}'...")

        try:
            # 1. Search the web
            web_results = search_web(query, max_results=5)

            if web_results.startswith("No web results") or web_results.startswith("Web search failed"):
                print(web_results)
                return

            print(f"Found results, creating concept node...")

            # 2. Embed the content
            content_to_embed = f"{query}\n\n{web_results}"
            print(f"Embedding content...")
            embedding = embed_text(content_to_embed, host=self.ollama_host)

            # 3. Create concept node
            client = self._get_client()
            concept_data = {
                "name": query,
                "kind": "concept",
                "source": "web_search",
                "content": web_results,
                "embedding": embedding,
            }

            concept_id = self._run_async(create_node(client, "concept", concept_data))
            print(f"Created concept: {concept_id}")

            # 4. Link to chunk if specified
            if link_to:
                edge_id = self._run_async(
                    create_edge(client, concept_id, "relates_to", link_to)
                )
                print(f"Linked to {link_to}: {edge_id}")

            # 5. Find and suggest related code chunks
            print(f"\nFinding related code chunks...")
            related = self._run_async(search_similar(client, embedding, limit=3))

            if related:
                print("\nRelated code chunks (use 'edge create' to link):")
                for r in related:
                    rid = r.get("id", "?")
                    rname = r.get("name", "?")
                    rkind = r.get("kind", "?")
                    rscore = r.get("score", 0)
                    print(f"  [{rid}] {rkind}: {rname} (score: {rscore:.4f})")
                print(f"\nExample: edge create {concept_id} relates_to {related[0].get('id')}")

        except Exception as e:
            print(f"Error adding concept: {e}")

    def _web_list(self, args: list[str]):
        """List stored concept nodes."""
        opts = self._parse_kv_args(args)
        limit = int(opts.get("limit", 10))

        try:
            client = self._get_client()
            concepts = self._run_async(list_nodes(client, "concept", limit))

            if not concepts:
                print("No concept nodes found. Use 'web add <query>' to add some.")
                return

            print(f"\nConcept nodes ({len(concepts)} found):")
            print("-" * 60)
            for c in concepts:
                cid = c.get("id", "?")
                name = c.get("name", "?")
                content = c.get("content", "")
                preview = content[:80] + "..." if len(content) > 80 else content
                preview = preview.replace("\n", " ")
                print(f"\n[{cid}] {name}")
                print(f"  {preview}")

        except Exception as e:
            print(f"Error listing concepts: {e}")

    # ==================== STATS COMMAND ====================

    def do_stats(self, arg: str):
        """Show database statistics.

        Usage:
            stats
        """
        try:
            client = self._get_client()
            stats = self._run_async(get_stats(client))

            print("\n╔══════════════════════════════════════╗")
            print("║     Knowledge Graph Statistics       ║")
            print("╠══════════════════════════════════════╣")
            print(f"║  Codebases:          {stats.get('codebases', 0):>10}    ║")
            print(f"║  Files:              {stats.get('files', 0):>10}    ║")
            print(f"║  Chunks:             {stats.get('chunks', 0):>10}    ║")
            print(f"║  Concepts (web):     {stats.get('concepts', 0):>10}    ║")
            print("╠══════════════════════════════════════╣")
            print(f"║  contains_file:      {stats.get('contains_file_edges', 0):>10}    ║")
            print(f"║  contains_chunk:     {stats.get('contains_chunk_edges', 0):>10}    ║")
            print(f"║  implements:         {stats.get('implements_edges', 0):>10}    ║")
            print(f"║  relates_to:         {stats.get('relates_to_edges', 0):>10}    ║")
            print("╚══════════════════════════════════════╝")
        except Exception as e:
            print(f"Stats error: {e}")

    # ==================== EXIT/HELP ====================

    def do_exit(self, arg: str):
        """Exit the REPL."""
        print("Goodbye!")
        return True

    def do_quit(self, arg: str):
        """Exit the REPL."""
        return self.do_exit(arg)

    def do_EOF(self, arg: str):
        """Handle Ctrl+D."""
        print()
        return self.do_exit(arg)

    def emptyline(self):
        """Don't repeat last command on empty line."""
        pass

    def default(self, line: str):
        """Handle unknown commands."""
        print(f"Unknown command: {line}")
        print("Type 'help' for available commands.")


def run_repl(
    surreal_url: str = "ws://localhost:8000/rpc",
    ollama_host: str = "http://localhost:11434",
):
    """Start the interactive REPL."""
    repl = GraphREPL(surreal_url=surreal_url, ollama_host=ollama_host)
    try:
        repl.cmdloop()
    except KeyboardInterrupt:
        print("\nGoodbye!")
