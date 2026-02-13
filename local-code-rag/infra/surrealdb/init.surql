USE NS code_rag DB code_rag;

-- ==========================
-- NODE TABLES
-- ==========================

DEFINE TABLE codebase SCHEMAFULL;
DEFINE FIELD name ON codebase TYPE string;
DEFINE FIELD path ON codebase TYPE string;
DEFINE FIELD language ON codebase TYPE string;
DEFINE FIELD indexed_at ON codebase TYPE datetime DEFAULT time::now();

DEFINE TABLE file SCHEMAFULL;
DEFINE FIELD path ON file TYPE string;
DEFINE FIELD relative_path ON file TYPE string;
DEFINE FIELD language ON file TYPE string;
DEFINE FIELD size_bytes ON file TYPE int;

DEFINE TABLE chunk SCHEMAFULL;
DEFINE FIELD kind ON chunk TYPE string;
DEFINE FIELD name ON chunk TYPE string;
DEFINE FIELD source_text ON chunk TYPE string;
DEFINE FIELD start_line ON chunk TYPE int;
DEFINE FIELD end_line ON chunk TYPE int;
DEFINE FIELD start_byte ON chunk TYPE int;
DEFINE FIELD end_byte ON chunk TYPE int;
DEFINE FIELD signature ON chunk TYPE option<string>;
DEFINE FIELD embedding ON chunk TYPE array<float>;
DEFINE FIELD metadata ON chunk TYPE object;

DEFINE INDEX chunk_embedding_idx ON chunk FIELDS embedding
    HNSW DIMENSION 768 DIST COSINE
    TYPE F32
    EFC 150
    M 12;

-- ==========================
-- EDGE TABLES (GRAPH)
-- ==========================

DEFINE TABLE contains_file TYPE RELATION IN codebase OUT file;

DEFINE TABLE contains_chunk TYPE RELATION IN file OUT chunk;

DEFINE TABLE implements TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD trait_name ON implements TYPE option<string>;

DEFINE TABLE calls TYPE RELATION IN chunk OUT chunk;

DEFINE TABLE imports TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD symbol ON imports TYPE string;

DEFINE TABLE references_type TYPE RELATION IN chunk OUT chunk;
DEFINE FIELD context ON references_type TYPE string;
