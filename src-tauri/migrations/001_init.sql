
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS folders (
    id          TEXT PRIMARY KEY,
    parent_id   TEXT REFERENCES folders(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL UNIQUE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);

CREATE TABLE IF NOT EXISTS notes (
    id              TEXT PRIMARY KEY,
    folder_id       TEXT REFERENCES folders(id) ON DELETE SET NULL,
    title           TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    content         TEXT NOT NULL DEFAULT '',
    content_format  TEXT NOT NULL DEFAULT 'markdown' CHECK (content_format IN ('markdown', 'richtext')),
    properties_json TEXT NOT NULL DEFAULT '{}',
    icon            TEXT,
    is_pinned       INTEGER NOT NULL DEFAULT 0 CHECK (is_pinned IN (0, 1)),
    is_archived     INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
    word_count      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id);
CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_notes_pinned ON notes(is_pinned);

CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title,
    content,
    content = 'notes',
    content_rowid = 'rowid'
);

CREATE TRIGGER IF NOT EXISTS notes_fts_after_insert AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_fts_after_delete AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES ('delete', old.rowid, old.title, old.content);
END;

CREATE TRIGGER IF NOT EXISTS notes_fts_after_update AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO notes_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;

CREATE TABLE IF NOT EXISTS note_versions (
    id          TEXT PRIMARY KEY,
    note_id     TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_note_versions_note ON note_versions(note_id, created_at DESC);

CREATE TABLE IF NOT EXISTS tags (
    id      TEXT PRIMARY KEY,
    name    TEXT NOT NULL UNIQUE,
    color   TEXT
);

CREATE TABLE IF NOT EXISTS note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);
CREATE INDEX IF NOT EXISTS idx_note_tags_tag ON note_tags(tag_id);

CREATE TABLE IF NOT EXISTS links (
    id              TEXT PRIMARY KEY,
    source_note_id  TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    target_note_id  TEXT REFERENCES notes(id) ON DELETE SET NULL,
    target_text     TEXT NOT NULL,
    is_broken       INTEGER NOT NULL DEFAULT 0 CHECK (is_broken IN (0, 1)),
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_links_source ON links(source_note_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_note_id);

CREATE TABLE IF NOT EXISTS attachments (
    id          TEXT PRIMARY KEY,
    note_id     TEXT REFERENCES notes(id) ON DELETE SET NULL,
    file_name   TEXT NOT NULL,
    file_path   TEXT NOT NULL,
    mime_type   TEXT,
    size_bytes  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX IF NOT EXISTS idx_attachments_note ON attachments(note_id);

CREATE TABLE IF NOT EXISTS _migrations (
    version     INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
