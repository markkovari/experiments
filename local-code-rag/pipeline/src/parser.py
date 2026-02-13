import tree_sitter_rust as ts_rust
from tree_sitter import Language, Parser, Node

from .models import CodeChunk

RUST_LANGUAGE = Language(ts_rust.language())

# Map tree-sitter node types to our chunk kinds
NODE_KIND_MAP = {
    "function_item": "function",
    "struct_item": "struct",
    "enum_item": "enum",
    "impl_item": "impl",
    "trait_item": "trait",
    "use_declaration": "use",
    "mod_item": "mod",
    "const_item": "const",
    "static_item": "static",
    "type_item": "type_alias",
    "macro_definition": "macro",
}

# Fields that hold the name identifier for each node type
NAME_FIELD_MAP = {
    "function_item": "name",
    "struct_item": "name",
    "enum_item": "name",
    "impl_item": "type",
    "trait_item": "name",
    "mod_item": "name",
    "const_item": "name",
    "static_item": "name",
    "type_item": "name",
    "macro_definition": "name",
}


def _extract_name(node: Node, source: bytes) -> str:
    """Extract the name/identifier from a node."""
    node_type = node.type

    if node_type == "use_declaration":
        return source[node.start_byte : node.end_byte].decode("utf-8").strip()

    field_name = NAME_FIELD_MAP.get(node_type)
    if field_name:
        name_node = node.child_by_field_name(field_name)
        if name_node:
            return source[name_node.start_byte : name_node.end_byte].decode("utf-8")

    return "<anonymous>"


def _extract_signature(node: Node, source: bytes) -> str | None:
    """Extract the signature (first line or declaration) of a node."""
    text = source[node.start_byte : node.end_byte].decode("utf-8")
    first_line = text.split("\n")[0].strip()

    if node.type in ("function_item", "struct_item", "enum_item", "trait_item"):
        # Return up to the opening brace
        brace_idx = first_line.find("{")
        if brace_idx > 0:
            return first_line[:brace_idx].strip()
        return first_line

    if node.type == "impl_item":
        # Return the impl header line
        return first_line.rstrip("{").strip()

    return None


def _extract_impl_methods(
    impl_node: Node, source: bytes, file_path: str, relative_path: str
) -> list[CodeChunk]:
    """Walk an impl_item's body to find function_items (methods)."""
    methods = []
    body = impl_node.child_by_field_name("body")
    if not body:
        return methods

    for child in body.children:
        if child.type == "function_item":
            name_node = child.child_by_field_name("name")
            name = (
                source[name_node.start_byte : name_node.end_byte].decode("utf-8")
                if name_node
                else "<anonymous>"
            )
            methods.append(
                CodeChunk(
                    kind="method",
                    name=name,
                    source_text=source[child.start_byte : child.end_byte].decode(
                        "utf-8"
                    ),
                    file_path=file_path,
                    relative_path=relative_path,
                    start_line=child.start_point[0] + 1,
                    end_line=child.end_point[0] + 1,
                    start_byte=child.start_byte,
                    end_byte=child.end_byte,
                    signature=_extract_signature(child, source),
                )
            )
    return methods


def _extract_impl_trait(impl_node: Node, source: bytes) -> str | None:
    """Check if an impl block implements a trait, return trait name if so."""
    trait_node = impl_node.child_by_field_name("trait")
    if trait_node:
        return source[trait_node.start_byte : trait_node.end_byte].decode("utf-8")
    return None


def parse_file(
    source_code: bytes, file_path: str, relative_path: str
) -> list[CodeChunk]:
    """Parse a Rust source file and extract semantic code chunks."""
    parser = Parser(RUST_LANGUAGE)
    tree = parser.parse(source_code)
    root = tree.root_node

    chunks = []

    for child in root.children:
        if child.type not in NODE_KIND_MAP:
            continue

        kind = NODE_KIND_MAP[child.type]
        name = _extract_name(child, source_code)
        source_text = source_code[child.start_byte : child.end_byte].decode("utf-8")
        signature = _extract_signature(child, source_code)

        chunk = CodeChunk(
            kind=kind,
            name=name,
            source_text=source_text,
            file_path=file_path,
            relative_path=relative_path,
            start_line=child.start_point[0] + 1,
            end_line=child.end_point[0] + 1,
            start_byte=child.start_byte,
            end_byte=child.end_byte,
            signature=signature,
        )

        # For impl blocks, extract methods and record trait info
        if child.type == "impl_item":
            chunk.children = _extract_impl_methods(
                child, source_code, file_path, relative_path
            )
            trait_name = _extract_impl_trait(child, source_code)
            if trait_name:
                chunk.metadata["trait"] = trait_name

        chunks.append(chunk)

        # Also add methods as top-level chunks so they get their own embeddings
        if chunk.children:
            chunks.extend(chunk.children)

    return chunks
