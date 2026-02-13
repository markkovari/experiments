"""Parsers for non-code project files: TOML, YAML, WIT."""

import re

from .models import CodeChunk


# ---------------------------------------------------------------------------
# TOML (Cargo.toml, wasmcloud.toml)
# ---------------------------------------------------------------------------

def parse_toml(source: str, file_path: str, relative_path: str) -> list[CodeChunk]:
    """Parse a TOML file into chunks by top-level tables and key groups."""
    chunks = []

    # Always include the whole file as a single chunk for context
    chunks.append(CodeChunk(
        kind="config_file",
        name=relative_path,
        source_text=source,
        file_path=file_path,
        relative_path=relative_path,
        start_line=1,
        end_line=source.count("\n") + 1,
        start_byte=0,
        end_byte=len(source.encode("utf-8")),
        signature=None,
        metadata={"format": "toml"},
    ))

    # Split into sections by [table] and [[array]] headers
    lines = source.split("\n")
    sections: list[tuple[str, int, list[str]]] = []
    current_header = "<root>"
    current_start = 1
    current_lines: list[str] = []

    for i, line in enumerate(lines):
        stripped = line.strip()
        match = re.match(r"^\[+([^\]]+)\]+", stripped)
        if match:
            # Flush previous section
            if current_lines:
                sections.append((current_header, current_start, current_lines))
            current_header = match.group(1).strip()
            current_start = i + 1
            current_lines = [line]
        else:
            current_lines.append(line)

    # Flush last section
    if current_lines:
        sections.append((current_header, current_start, current_lines))

    for header, start_line, section_lines in sections:
        text = "\n".join(section_lines).strip()
        if not text or header == "<root>" and not any(
            "=" in l for l in section_lines
        ):
            continue

        end_line = start_line + len(section_lines) - 1
        encoded = text.encode("utf-8")
        kind = "toml_table" if header != "<root>" else "toml_section"

        chunks.append(CodeChunk(
            kind=kind,
            name=f"[{header}]" if header != "<root>" else relative_path,
            source_text=text,
            file_path=file_path,
            relative_path=relative_path,
            start_line=start_line,
            end_line=end_line,
            start_byte=0,
            end_byte=len(encoded),
            signature=f"[{header}]" if header != "<root>" else None,
            metadata={"format": "toml", "table": header},
        ))

    return chunks


# ---------------------------------------------------------------------------
# YAML (wadm.yaml)
# ---------------------------------------------------------------------------

def parse_yaml(source: str, file_path: str, relative_path: str) -> list[CodeChunk]:
    """Parse a YAML file into chunks by top-level keys."""
    chunks = []

    # Whole file as one chunk
    chunks.append(CodeChunk(
        kind="config_file",
        name=relative_path,
        source_text=source,
        file_path=file_path,
        relative_path=relative_path,
        start_line=1,
        end_line=source.count("\n") + 1,
        start_byte=0,
        end_byte=len(source.encode("utf-8")),
        signature=None,
        metadata={"format": "yaml"},
    ))

    # Split by top-level keys (lines starting with a non-space, non-comment, non-dash char followed by ':')
    lines = source.split("\n")
    sections: list[tuple[str, int, list[str]]] = []
    current_key = ""
    current_start = 1
    current_lines: list[str] = []

    for i, line in enumerate(lines):
        # Top-level key: starts at column 0, not a comment, not a list item, contains ':'
        if line and not line[0].isspace() and not line.startswith("#") and not line.startswith("---") and ":" in line:
            if current_lines and current_key:
                sections.append((current_key, current_start, current_lines))
            current_key = line.split(":")[0].strip()
            current_start = i + 1
            current_lines = [line]
        else:
            current_lines.append(line)

    if current_lines and current_key:
        sections.append((current_key, current_start, current_lines))

    for key, start_line, section_lines in sections:
        text = "\n".join(section_lines).strip()
        if not text:
            continue

        end_line = start_line + len(section_lines) - 1
        encoded = text.encode("utf-8")

        chunks.append(CodeChunk(
            kind="yaml_mapping",
            name=key,
            source_text=text,
            file_path=file_path,
            relative_path=relative_path,
            start_line=start_line,
            end_line=end_line,
            start_byte=0,
            end_byte=len(encoded),
            signature=f"{key}:",
            metadata={"format": "yaml", "key": key},
        ))

    return chunks


# ---------------------------------------------------------------------------
# WIT (world.wit)
# ---------------------------------------------------------------------------

def parse_wit(source: str, file_path: str, relative_path: str) -> list[CodeChunk]:
    """Parse a WIT file into chunks by package, world, interface, and type declarations."""
    chunks = []

    # Whole file
    chunks.append(CodeChunk(
        kind="config_file",
        name=relative_path,
        source_text=source,
        file_path=file_path,
        relative_path=relative_path,
        start_line=1,
        end_line=source.count("\n") + 1,
        start_byte=0,
        end_byte=len(source.encode("utf-8")),
        signature=None,
        metadata={"format": "wit"},
    ))

    lines = source.split("\n")

    # Extract package declaration
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("package "):
            pkg_name = stripped.rstrip(";").replace("package ", "").strip()
            chunks.append(CodeChunk(
                kind="wit_interface",
                name=pkg_name,
                source_text=stripped,
                file_path=file_path,
                relative_path=relative_path,
                start_line=i + 1,
                end_line=i + 1,
                start_byte=0,
                end_byte=len(stripped.encode("utf-8")),
                signature=f"package {pkg_name}",
                metadata={"format": "wit", "declaration": "package"},
            ))

    # Extract world/interface blocks with their contents
    block_pattern = re.compile(r"^\s*(world|interface)\s+(\S+)\s*\{")
    i = 0
    while i < len(lines):
        match = block_pattern.match(lines[i])
        if match:
            block_type = match.group(1)  # "world" or "interface"
            block_name = match.group(2)
            start_line = i + 1
            block_lines = [lines[i]]
            brace_depth = lines[i].count("{") - lines[i].count("}")
            i += 1

            while i < len(lines) and brace_depth > 0:
                block_lines.append(lines[i])
                brace_depth += lines[i].count("{") - lines[i].count("}")
                i += 1

            text = "\n".join(block_lines)
            kind = "wit_world" if block_type == "world" else "wit_interface"

            chunks.append(CodeChunk(
                kind=kind,
                name=block_name,
                source_text=text,
                file_path=file_path,
                relative_path=relative_path,
                start_line=start_line,
                end_line=start_line + len(block_lines) - 1,
                start_byte=0,
                end_byte=len(text.encode("utf-8")),
                signature=f"{block_type} {block_name}",
                metadata={"format": "wit", "declaration": block_type},
            ))

            # Extract export/import/use/type statements inside the block
            for j, bline in enumerate(block_lines):
                stripped = bline.strip()
                for keyword in ("export", "import", "use", "type", "record", "variant", "enum", "flags", "resource"):
                    if stripped.startswith(f"{keyword} "):
                        stmt_name = stripped.rstrip(";").rstrip("{")
                        chunks.append(CodeChunk(
                            kind="wit_function" if keyword in ("export", "import") else "wit_type",
                            name=stmt_name,
                            source_text=stripped,
                            file_path=file_path,
                            relative_path=relative_path,
                            start_line=start_line + j,
                            end_line=start_line + j,
                            start_byte=0,
                            end_byte=len(stripped.encode("utf-8")),
                            signature=stmt_name,
                            metadata={"format": "wit", "declaration": keyword, "parent": block_name},
                        ))
                        break
        else:
            i += 1

    return chunks


# ---------------------------------------------------------------------------
# Dispatcher
# ---------------------------------------------------------------------------

# Map file extensions/names to their parsers and language labels
FILE_PARSERS = {
    ".toml": ("toml", parse_toml),
    ".yaml": ("yaml", parse_yaml),
    ".yml": ("yaml", parse_yaml),
    ".wit": ("wit", parse_wit),
}


def parse_config_file(
    source: str, file_path: str, relative_path: str
) -> tuple[str, list[CodeChunk]]:
    """Parse a config file based on its extension. Returns (language, chunks)."""
    for ext, (lang, parser_fn) in FILE_PARSERS.items():
        if relative_path.endswith(ext):
            return lang, parser_fn(source, file_path, relative_path)
    return "unknown", []
