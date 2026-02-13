USE NS code_rag DB code_rag;

-- ==========================
-- NODE TABLES
-- ==========================

DEFINE TABLE OVERWRITE codebase SCHEMAFULL;
DEFINE FIELD OVERWRITE name ON codebase TYPE string;
DEFINE FIELD OVERWRITE path ON codebase TYPE string;
DEFINE FIELD OVERWRITE language ON codebase TYPE string;
DEFINE FIELD OVERWRITE indexed_at ON codebase TYPE datetime DEFAULT time::now();

DEFINE TABLE OVERWRITE file SCHEMAFULL;
DEFINE FIELD OVERWRITE path ON file TYPE string;
DEFINE FIELD OVERWRITE relative_path ON file TYPE string;
DEFINE FIELD OVERWRITE language ON file TYPE string;
DEFINE FIELD OVERWRITE size_bytes ON file TYPE int;

DEFINE TABLE OVERWRITE chunk SCHEMAFULL;
DEFINE FIELD OVERWRITE kind ON chunk TYPE string;
DEFINE FIELD OVERWRITE name ON chunk TYPE string;
DEFINE FIELD OVERWRITE source_text ON chunk TYPE string;
DEFINE FIELD OVERWRITE start_line ON chunk TYPE int;
DEFINE FIELD OVERWRITE end_line ON chunk TYPE int;
DEFINE FIELD OVERWRITE start_byte ON chunk TYPE int;
DEFINE FIELD OVERWRITE end_byte ON chunk TYPE int;
DEFINE FIELD OVERWRITE signature ON chunk TYPE option<string>;
DEFINE FIELD OVERWRITE embedding ON chunk TYPE array<float>;
DEFINE FIELD OVERWRITE metadata ON chunk TYPE object;

DEFINE INDEX OVERWRITE chunk_embedding_idx ON chunk FIELDS embedding
    HNSW DIMENSION 768 DIST COSINE
    TYPE F32
    EFC 150
    M 12;

-- ==========================
-- EDGE TABLES (GRAPH)
-- ==========================

DEFINE TABLE OVERWRITE contains_file TYPE RELATION IN codebase OUT file;

DEFINE TABLE OVERWRITE contains_chunk TYPE RELATION IN file OUT chunk;

DEFINE TABLE OVERWRITE implements TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD OVERWRITE trait_name ON implements TYPE option<string>;

DEFINE TABLE OVERWRITE calls TYPE RELATION IN chunk OUT chunk;

DEFINE TABLE OVERWRITE imports TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD OVERWRITE symbol ON imports TYPE string;

DEFINE TABLE OVERWRITE references_type TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD OVERWRITE context ON references_type TYPE string;
