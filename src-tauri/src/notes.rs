use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::link_parser;
use crate::links;
use crate::models::{
    CreateNoteInput, Note, NoteSummary, NoteVersion, SearchHit, UpdateNoteContentInput,
};
use crate::slug::slugify;

const NOTE_COLUMNS: &str = "id, folder_id, title, slug, content, content_format, properties_json, icon, color, is_pinned, is_archived, word_count, created_at, updated_at";

fn row_to_note(row: &rusqlite::Row) -> rusqlite::Result<Note> {
    let properties_json: String = row.get(6)?;
    let properties = serde_json::from_str(&properties_json).unwrap_or(serde_json::json!({}));
    Ok(Note {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        slug: row.get(3)?,
        content: row.get(4)?,
        content_format: row.get(5)?,
        properties,
        icon: row.get(7)?,
        color: row.get(8)?,
        is_pinned: row.get::<_, i64>(9)? != 0,
        is_archived: row.get::<_, i64>(10)? != 0,
        word_count: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_summary(row: &rusqlite::Row) -> rusqlite::Result<NoteSummary> {
    let raw_preview: String = row.get(6)?;
    Ok(NoteSummary {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        slug: row.get(3)?,
        icon: row.get(4)?,
        color: row.get(5)?,
        content_preview: clean_preview(&raw_preview),
        is_pinned: row.get::<_, i64>(7)? != 0,
        is_archived: row.get::<_, i64>(8)? != 0,
        updated_at: row.get(9)?,
    })
}
fn clean_preview(raw: &str) -> String {
    const MAX_LEN: usize = 120;

    let de_linked = link_parser::strip_wiki_link_syntax(raw);
    let no_heading_markers = de_linked.trim_start().trim_start_matches('#').trim_start();
    let collapsed: String = no_heading_markers
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if collapsed.chars().count() <= MAX_LEN {
        collapsed
    } else {
        let truncated: String = collapsed.chars().take(MAX_LEN).collect();
        format!("{}…", truncated.trim_end())
    }
}

fn word_count(content: &str) -> i64 {
    content.split_whitespace().count() as i64
}
fn unique_slug(conn: &Connection, title: &str, exclude_note_id: Option<&str>) -> AppResult<String> {
    let base = slugify(title);
    let mut candidate = base.clone();
    let mut suffix = 2;

    loop {
        let taken: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM notes WHERE slug = ?1 AND id IS NOT ?2)",
            (&candidate, exclude_note_id),
            |row| row.get(0),
        )?;
        if !taken {
            return Ok(candidate);
        }
        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
}

pub fn get_note(conn: &Connection, note_id: &str) -> AppResult<Note> {
    conn.query_row(
        &format!("SELECT {NOTE_COLUMNS} FROM notes WHERE id = ?1"),
        [note_id],
        row_to_note,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("note {note_id}")),
        other => AppError::from(other),
    })
}

pub fn create_note(conn: &Connection, input: CreateNoteInput) -> AppResult<Note> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("note title cannot be empty".into()));
    }

    let id = Uuid::new_v4().to_string();
    let slug = unique_slug(conn, title, None)?;
    let content = input.content.unwrap_or_default();
    let content_format = input
        .content_format
        .unwrap_or_else(|| "markdown".to_string());
    let wc = word_count(&content);

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO notes (id, folder_id, title, slug, content, content_format, word_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &id,
            &input.folder_id,
            title,
            &slug,
            &content,
            &content_format,
            wc,
        ),
    )?;
    links::sync_links_for_note(&tx, &id, &content)?;
    links::resolve_broken_links_targeting(&tx, &id, title, &slug)?;
    tx.commit()?;

    get_note(conn, &id)
}

pub fn update_note_content(conn: &Connection, input: UpdateNoteContentInput) -> AppResult<Note> {
    let existing = get_note(conn, &input.note_id)?;
    let wc = word_count(&input.content);

    let tx = conn.unchecked_transaction()?;
    if input.snapshot_previous {
        tx.execute(
            "INSERT INTO note_versions (id, note_id, title, content) VALUES (?1, ?2, ?3, ?4)",
            (
                Uuid::new_v4().to_string(),
                &input.note_id,
                &existing.title,
                &existing.content,
            ),
        )?;
    }
    tx.execute(
        "UPDATE notes SET content = ?1, word_count = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?3",
        (&input.content, wc, &input.note_id),
    )?;
    links::sync_links_for_note(&tx, &input.note_id, &input.content)?;
    tx.commit()?;

    get_note(conn, &input.note_id)
}

pub fn rename_note(conn: &Connection, note_id: &str, new_title: &str) -> AppResult<Note> {
    let trimmed = new_title.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("note title cannot be empty".into()));
    }

    let tx = conn.unchecked_transaction()?;
    let new_slug = unique_slug(&tx, trimmed, Some(note_id))?;
    tx.execute(
        "UPDATE notes SET title = ?1, slug = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?3",
        (trimmed, &new_slug, note_id),
    )?;
    links::resolve_broken_links_targeting(&tx, note_id, trimmed, &new_slug)?;
    tx.commit()?;

    get_note(conn, note_id)
}

pub fn move_note(conn: &Connection, note_id: &str, folder_id: Option<&str>) -> AppResult<Note> {
    conn.execute(
        "UPDATE notes SET folder_id = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (folder_id, note_id),
    )?;
    get_note(conn, note_id)
}

pub fn set_pinned(conn: &Connection, note_id: &str, pinned: bool) -> AppResult<Note> {
    conn.execute(
        "UPDATE notes SET is_pinned = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (pinned, note_id),
    )?;
    get_note(conn, note_id)
}

pub fn set_archived(conn: &Connection, note_id: &str, archived: bool) -> AppResult<Note> {
    conn.execute(
        "UPDATE notes SET is_archived = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (archived, note_id),
    )?;
    get_note(conn, note_id)
}
pub fn set_color(conn: &Connection, note_id: &str, color: Option<&str>) -> AppResult<Note> {
    conn.execute(
        "UPDATE notes SET color = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (color, note_id),
    )?;
    get_note(conn, note_id)
}

pub fn set_properties(
    conn: &Connection,
    note_id: &str,
    properties: &serde_json::Value,
) -> AppResult<Note> {
    let json = serde_json::to_string(properties)?;
    conn.execute(
        "UPDATE notes SET properties_json = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (json, note_id),
    )?;
    get_note(conn, note_id)
}

pub fn delete_note(conn: &Connection, note_id: &str) -> AppResult<()> {
    let exists: Option<String> = conn
        .query_row("SELECT id FROM notes WHERE id = ?1", [note_id], |r| {
            r.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("note {note_id}")));
    }

    let tx = conn.unchecked_transaction()?;
    links::mark_links_broken_for_target(&tx, note_id)?;
    tx.execute("DELETE FROM notes WHERE id = ?1", [note_id])?;
    tx.commit()?;
    Ok(())
}

pub fn list_notes(
    conn: &Connection,
    folder_id: Option<&str>,
    include_archived: bool,
) -> AppResult<Vec<NoteSummary>> {
    let columns = "id, folder_id, title, slug, icon, color, substr(content, 1, 200), is_pinned, is_archived, updated_at";
    let sql = match folder_id {
        Some(_) => format!(
            "SELECT {columns} FROM notes WHERE folder_id = ?1 AND (is_archived = 0 OR ?2)
             ORDER BY is_pinned DESC, updated_at DESC"
        ),
        None => format!(
            "SELECT {columns} FROM notes WHERE (is_archived = 0 OR ?2)
             ORDER BY is_pinned DESC, updated_at DESC"
        ),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = match folder_id {
        Some(fid) => stmt.query_map((fid, include_archived), row_to_summary)?,
        None => stmt.query_map((rusqlite::types::Null, include_archived), row_to_summary)?,
    };
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_note_versions(conn: &Connection, note_id: &str) -> AppResult<Vec<NoteVersion>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_id, title, content, created_at FROM note_versions
         WHERE note_id = ?1 ORDER BY created_at DESC, rowid DESC",
    )?;
    let rows = stmt.query_map([note_id], |row| {
        Ok(NoteVersion {
            id: row.get(0)?,
            note_id: row.get(1)?,
            title: row.get(2)?,
            content: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
pub fn create_snapshot(conn: &Connection, note_id: &str) -> AppResult<NoteVersion> {
    let note = get_note(conn, note_id)?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO note_versions (id, note_id, title, content) VALUES (?1, ?2, ?3, ?4)",
        (&id, note_id, &note.title, &note.content),
    )?;
    conn.query_row(
        "SELECT id, note_id, title, content, created_at FROM note_versions WHERE id = ?1",
        [&id],
        |row| {
            Ok(NoteVersion {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )
    .map_err(AppError::from)
}
pub fn restore_note_version(conn: &Connection, version_id: &str) -> AppResult<Note> {
    let (note_id, title, content): (String, String, String) = conn
        .query_row(
            "SELECT note_id, title, content FROM note_versions WHERE id = ?1",
            [version_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("note version {version_id}"))
            }
            other => AppError::from(other),
        })?;

    // Restores are undoable: the current content is snapshotted before replacement.
    update_note_content(
        conn,
        UpdateNoteContentInput {
            note_id: note_id.clone(),
            content,
            snapshot_previous: true,
        },
    )?;
    rename_note(conn, &note_id, &title)
}
pub fn search_notes(conn: &Connection, query: &str, limit: i64) -> AppResult<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }
    // FTS5 treats punctuation as syntax; quote terms so plain user input stays safe.
    let sanitized: String = trimmed
        .split_whitespace()
        .map(|term| format!("\"{}\"", term.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" ");

    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, n.slug,
                snippet(notes_fts, 1, '<mark>', '</mark>', '…', 12) AS snippet,
                bm25(notes_fts) AS rank
         FROM notes_fts
         JOIN notes n ON n.rowid = notes_fts.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY rank ASC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map((&sanitized, limit), |row| {
        Ok(SearchHit {
            note_id: row.get(0)?,
            title: row.get(1)?,
            slug: row.get(2)?,
            snippet: row.get(3)?,
            rank: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::run_migrations(&conn).unwrap();
        conn
    }

    fn make(conn: &Connection, title: &str, content: &str) -> Note {
        create_note(
            conn,
            CreateNoteInput {
                title: title.to_string(),
                folder_id: None,
                content: Some(content.to_string()),
                content_format: None,
            },
        )
        .unwrap()
    }

    #[test]
    fn creates_a_note_with_generated_slug() {
        let conn = setup();
        let note = make(&conn, "My First Note", "");
        assert_eq!(note.slug, "my-first-note");
        assert_eq!(note.word_count, 0);
    }

    #[test]
    fn duplicate_titles_get_distinct_slugs() {
        let conn = setup();
        let a = make(&conn, "Weekly Review", "");
        let b = make(&conn, "Weekly Review", "");
        assert_ne!(a.slug, b.slug);
        assert_eq!(b.slug, "weekly-review-2");
    }

    #[test]
    fn rejects_blank_titles() {
        let conn = setup();
        let result = create_note(
            &conn,
            CreateNoteInput {
                title: "   ".into(),
                folder_id: None,
                content: None,
                content_format: None,
            },
        );
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn updating_content_recomputes_word_count_and_links() {
        let conn = setup();
        let target = make(&conn, "Target", "");
        let note = make(&conn, "Source", "");

        let updated = update_note_content(
            &conn,
            UpdateNoteContentInput {
                note_id: note.id.clone(),
                content: format!("Five simple words here, linking to [[{}]].", target.title),
                snapshot_previous: false,
            },
        )
        .unwrap();

        assert_eq!(updated.word_count, 7);
        let backlinks = links::get_backlinks(&conn, &target.id).unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_note_id, note.id);
    }

    #[test]
    fn snapshot_on_update_preserves_history() {
        let conn = setup();
        let note = make(&conn, "Journal", "first draft");

        update_note_content(
            &conn,
            UpdateNoteContentInput {
                note_id: note.id.clone(),
                content: "second draft".into(),
                snapshot_previous: true,
            },
        )
        .unwrap();

        let versions = list_note_versions(&conn, &note.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].content, "first draft");
    }

    #[test]
    fn create_snapshot_captures_current_content_not_previous() {
        let conn = setup();
        let note = make(&conn, "Journal", "first draft");

        update_note_content(
            &conn,
            UpdateNoteContentInput {
                note_id: note.id.clone(),
                content: "second draft".into(),
                snapshot_previous: false,
            },
        )
        .unwrap();

        let snapshot = create_snapshot(&conn, &note.id).unwrap();
        assert_eq!(
            snapshot.content, "second draft",
            "snapshot must capture the CURRENT content"
        );

        let versions = list_note_versions(&conn, &note.id).unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].content, "second draft");
    }

    #[test]
    fn create_snapshot_can_be_called_multiple_times() {
        let conn = setup();
        let note = make(&conn, "Journal", "v1");
        create_snapshot(&conn, &note.id).unwrap();

        update_note_content(
            &conn,
            UpdateNoteContentInput {
                note_id: note.id.clone(),
                content: "v2".into(),
                snapshot_previous: false,
            },
        )
        .unwrap();
        create_snapshot(&conn, &note.id).unwrap();

        let versions = list_note_versions(&conn, &note.id).unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].content, "v2");
        assert_eq!(versions[1].content, "v1");
    }

    #[test]
    fn create_snapshot_fails_clearly_for_nonexistent_note() {
        let conn = setup();
        let result = create_snapshot(&conn, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn restoring_a_version_brings_back_old_content() {
        let conn = setup();
        let note = make(&conn, "Journal", "first draft");
        update_note_content(
            &conn,
            UpdateNoteContentInput {
                note_id: note.id.clone(),
                content: "second draft".into(),
                snapshot_previous: true,
            },
        )
        .unwrap();

        let versions = list_note_versions(&conn, &note.id).unwrap();
        restore_note_version(&conn, &versions[0].id).unwrap();

        let restored = get_note(&conn, &note.id).unwrap();
        assert_eq!(restored.content, "first draft");
    }

    #[test]
    fn deleting_a_note_removes_it_and_breaks_incoming_links() {
        let conn = setup();
        let target = make(&conn, "Target", "");
        let note = make(&conn, "Source", &format!("[[{}]]", target.title));

        delete_note(&conn, &target.id).unwrap();

        assert!(matches!(
            get_note(&conn, &target.id),
            Err(AppError::NotFound(_))
        ));
        let outgoing = links::get_outgoing_links(&conn, &note.id).unwrap();
        assert!(outgoing[0].is_broken);
    }

    #[test]
    fn full_text_search_finds_matching_notes() {
        let conn = setup();
        make(
            &conn,
            "Quarterly Planning",
            "Roadmap for the next quarter, focused on growth.",
        );
        make(&conn, "Recipe Ideas", "Pasta with garlic and olive oil.");

        let hits = search_notes(&conn, "quarter", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Quarterly Planning");
    }

    #[test]
    fn search_with_special_characters_does_not_error() {
        let conn = setup();
        make(&conn, "C++ Notes", "Memory management in C++.");
        let hits = search_notes(&conn, "C++ -pointer \"quote", 10).unwrap();
        let _ = hits;
    }

    #[test]
    fn list_notes_excludes_archived_by_default() {
        let conn = setup();
        let note = make(&conn, "Old Note", "");
        set_archived(&conn, &note.id, true).unwrap();
        make(&conn, "Active Note", "");

        let visible = list_notes(&conn, None, false).unwrap();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Active Note");

        let all = list_notes(&conn, None, true).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn clean_preview_strips_wiki_links_and_collapses_whitespace() {
        assert_eq!(
            clean_preview("Met with [[Alice]] about the\n\n[[Q3 Roadmap|roadmap]] today."),
            "Met with Alice about the roadmap today.",
        );
    }

    #[test]
    fn clean_preview_strips_leading_heading_marker() {
        assert_eq!(
            clean_preview("# Project Kickoff\nNotes from the meeting."),
            "Project Kickoff Notes from the meeting."
        );
        assert_eq!(
            clean_preview("## Sub Heading text here"),
            "Sub Heading text here"
        );
    }

    #[test]
    fn clean_preview_truncates_long_text_with_ellipsis() {
        let long = "word ".repeat(40);
        let preview = clean_preview(&long);
        assert!(preview.ends_with('…'));
        assert!(preview.chars().count() <= 121);
    }

    #[test]
    fn clean_preview_handles_empty_content() {
        assert_eq!(clean_preview(""), "");
    }

    #[test]
    fn list_notes_includes_a_readable_content_preview() {
        let conn = setup();
        make(
            &conn,
            "Meeting Notes",
            "Discussed [[Project Alpha]] timeline and budget.",
        );

        let notes = list_notes(&conn, None, false).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(
            notes[0].content_preview,
            "Discussed Project Alpha timeline and budget."
        );
    }

    #[test]
    fn set_color_overrides_and_clears() {
        let conn = setup();
        let note = make(&conn, "Colored note", "");
        assert_eq!(note.color, None);

        let colored = set_color(&conn, &note.id, Some("#ff0055")).unwrap();
        assert_eq!(colored.color, Some("#ff0055".to_string()));

        let cleared = set_color(&conn, &note.id, None).unwrap();
        assert_eq!(cleared.color, None);
    }
}
