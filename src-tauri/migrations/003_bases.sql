
CREATE TABLE IF NOT EXISTS bases (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS base_notes (
    base_id     TEXT NOT NULL REFERENCES bases(id) ON DELETE CASCADE,
    note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (base_id, note_id)
);
CREATE INDEX IF NOT EXISTS idx_base_notes_note ON base_notes(note_id);
CREATE INDEX IF NOT EXISTS idx_base_notes_base ON base_notes(base_id);
