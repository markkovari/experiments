CREATE TABLE IF NOT EXISTS items (
    id      INTEGER PRIMARY KEY,
    name    TEXT    NOT NULL,
    qty     INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1
);

-- 3. pessimistic lock / lease: at most one lock row per item
CREATE TABLE IF NOT EXISTS locks (
    item_id    INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    token      TEXT    NOT NULL,
    expires_at INTEGER NOT NULL -- unix seconds
);

-- 6. event sourcing: the primary key rejects two writers appending the same version
CREATE TABLE IF NOT EXISTS events (
    stream_id TEXT    NOT NULL,
    version   INTEGER NOT NULL,
    type      TEXT    NOT NULL,
    data      TEXT    NOT NULL,
    PRIMARY KEY (stream_id, version)
);

-- 7. command queue
CREATE TABLE IF NOT EXISTS commands (
    id      INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL,
    payload TEXT    NOT NULL,
    status  TEXT    NOT NULL DEFAULT 'pending', -- pending | done | failed
    error   TEXT
);

-- 2 + 5. Every committed version of every item. Triggers fill it, so all endpoints are covered.
-- shortcut: history grows forever; prune old versions if items get many writes.
CREATE TABLE IF NOT EXISTS item_history (
    item_id INTEGER NOT NULL,
    version INTEGER NOT NULL,
    name    TEXT    NOT NULL,
    qty     INTEGER NOT NULL,
    at      INTEGER NOT NULL DEFAULT (unixepoch()), -- unix seconds, drives Last-Modified
    PRIMARY KEY (item_id, version)
);
CREATE TRIGGER IF NOT EXISTS items_history_insert AFTER INSERT ON items BEGIN
    INSERT INTO item_history (item_id, version, name, qty) VALUES (new.id, new.version, new.name, new.qty);
END;
CREATE TRIGGER IF NOT EXISTS items_history_update AFTER UPDATE ON items BEGIN
    INSERT INTO item_history (item_id, version, name, qty) VALUES (new.id, new.version, new.name, new.qty);
END;

-- 8. CRDT: PN-counter (one row per replica) + last-writer-wins register
CREATE TABLE IF NOT EXISTS crdt_counters (
    key     TEXT    NOT NULL,
    replica TEXT    NOT NULL,
    p       INTEGER NOT NULL, -- total increments this replica has ever made
    n       INTEGER NOT NULL, -- total decrements
    PRIMARY KEY (key, replica)
);
CREATE TABLE IF NOT EXISTS crdt_registers (
    key     TEXT    PRIMARY KEY,
    value   TEXT    NOT NULL,
    ts      INTEGER NOT NULL,
    replica TEXT    NOT NULL
);

-- 9. OT: text documents and the transformed ops applied at each version
CREATE TABLE IF NOT EXISTS ot_docs (
    id      INTEGER PRIMARY KEY,
    text    TEXT    NOT NULL,
    version INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS ot_ops (
    doc_id  INTEGER NOT NULL,
    version INTEGER NOT NULL,
    ops     TEXT    NOT NULL, -- JSON array of ops, in the coordinates of version - 1
    PRIMARY KEY (doc_id, version)
);
