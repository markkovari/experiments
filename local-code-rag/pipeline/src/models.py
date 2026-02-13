from dataclasses import dataclass, field
from typing import Optional


@dataclass
class CodeChunk:
    kind: str  # "function", "struct", "impl", "use", "enum", "trait", "const", "static", "mod", "method", "macro", "toml_section", "toml_table", "yaml_document", "yaml_mapping", "wit_interface", "wit_world", "wit_type", "wit_function", "config_file"
    name: str
    source_text: str
    file_path: str
    relative_path: str
    start_line: int
    end_line: int
    start_byte: int
    end_byte: int
    signature: Optional[str] = None
    metadata: dict = field(default_factory=dict)
    embedding: list[float] = field(default_factory=list)
    children: list["CodeChunk"] = field(default_factory=list)


@dataclass
class FileInfo:
    path: str
    relative_path: str
    language: str
    size_bytes: int
    chunks: list[CodeChunk] = field(default_factory=list)


@dataclass
class CodebaseInfo:
    name: str
    path: str
    language: str
    files: list[FileInfo] = field(default_factory=list)
