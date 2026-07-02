use rusqlite::Connection;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{NoteTagAssignment, Tag};

fn row_to_tag(row: &rusqlite::Row) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
    })
}

pub fn list_tags(conn: &Connection) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name ASC")?;
    let rows = stmt.query_map([], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
fn fold_case(s: &str) -> String {
    s.chars()
        .map(|c| if c == '\u{0130}' { 'i' } else { c })
        .collect::<String>()
        .to_lowercase()
}

pub fn get_or_create_tag(conn: &Connection, name: &str) -> AppResult<Tag> {
    let trimmed = name.trim().trim_start_matches('#');
    if trimmed.is_empty() {
        return Err(AppError::Validation("tag name cannot be empty".into()));
    }

    let normalized = fold_case(trimmed);
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let existing_name: String = row.get(1)?;
        if fold_case(&existing_name) == normalized {
            return Ok(Tag {
                id: row.get(0)?,
                name: existing_name,
                color: row.get(2)?,
            });
        }
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        (&id, trimmed),
    )?;
    Ok(Tag {
        id,
        name: trimmed.to_string(),
        color: None,
    })
}

pub fn set_tag_color(conn: &Connection, tag_id: &str, color: Option<&str>) -> AppResult<Tag> {
    conn.execute("UPDATE tags SET color = ?1 WHERE id = ?2", (color, tag_id))?;
    conn.query_row(
        "SELECT id, name, color FROM tags WHERE id = ?1",
        [tag_id],
        row_to_tag,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("tag {tag_id}")),
        other => AppError::from(other),
    })
}

pub fn add_tag_to_note(conn: &Connection, note_id: &str, tag_name: &str) -> AppResult<Tag> {
    let tag = get_or_create_tag(conn, tag_name)?;
    conn.execute(
        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
        (note_id, &tag.id),
    )?;
    Ok(tag)
}

pub fn remove_tag_from_note(conn: &Connection, note_id: &str, tag_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
        (note_id, tag_id),
    )?;
    Ok(())
}

pub fn list_tags_for_note(conn: &Connection, note_id: &str) -> AppResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         JOIN note_tags nt ON nt.tag_id = t.id
         WHERE nt.note_id = ?1 ORDER BY t.name ASC",
    )?;
    let rows = stmt.query_map([note_id], row_to_tag)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
pub fn list_notes_for_tag(conn: &Connection, tag_id: &str) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT nt.note_id FROM note_tags nt
         JOIN notes n ON n.id = nt.note_id
         WHERE nt.tag_id = ?1 ORDER BY n.updated_at DESC",
    )?;
    let rows = stmt.query_map([tag_id], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}
pub fn list_all_note_tags(conn: &Connection) -> AppResult<Vec<NoteTagAssignment>> {
    let mut stmt = conn.prepare(
        "SELECT nt.note_id, t.id, t.name, t.color
         FROM note_tags nt
         JOIN tags t ON t.id = nt.tag_id
         ORDER BY nt.note_id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(NoteTagAssignment {
            note_id: row.get(0)?,
            tag_id: row.get(1)?,
            tag_name: row.get(2)?,
            color: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CreateNoteInput;
    use crate::notes::create_note;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn get_or_create_tag_is_case_insensitive() {
        let conn = setup();
        let a = get_or_create_tag(&conn, "Work").unwrap();
        let b = get_or_create_tag(&conn, "work").unwrap();
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn get_or_create_tag_folds_turkish_casing_too() {
        let conn = setup();
        let a = get_or_create_tag(&conn, "İş").unwrap();
        let b = get_or_create_tag(&conn, "iş").unwrap();
        assert_eq!(a.id, b.id);

        let all = list_tags(&conn).unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn strips_leading_hash_from_tag_names() {
        let conn = setup();
        let tag = get_or_create_tag(&conn, "#project").unwrap();
        assert_eq!(tag.name, "project");
    }

    #[test]
    fn tagging_and_listing_notes_for_a_tag() {
        let conn = setup();
        let note = create_note(
            &conn,
            CreateNoteInput {
                title: "Tagged Note".into(),
                folder_id: None,
                content: None,
                content_format: None,
            },
        )
        .unwrap();

        let tag = add_tag_to_note(&conn, &note.id, "research").unwrap();
        let notes_for_tag = list_notes_for_tag(&conn, &tag.id).unwrap();
        assert_eq!(notes_for_tag, vec![note.id.clone()]);

        let tags_for_note = list_tags_for_note(&conn, &note.id).unwrap();
        assert_eq!(tags_for_note.len(), 1);
        assert_eq!(tags_for_note[0].name, "research");

        remove_tag_from_note(&conn, &note.id, &tag.id).unwrap();
        assert!(list_tags_for_note(&conn, &note.id).unwrap().is_empty());
    }

    #[test]
    fn list_all_note_tags_includes_color_for_every_assignment() {
        let conn = setup();
        let note_a = create_note(
            &conn,
            CreateNoteInput {
                title: "A".into(),
                folder_id: None,
                content: None,
                content_format: None,
            },
        )
        .unwrap();
        let note_b = create_note(
            &conn,
            CreateNoteInput {
                title: "B".into(),
                folder_id: None,
                content: None,
                content_format: None,
            },
        )
        .unwrap();

        let work_tag = add_tag_to_note(&conn, &note_a.id, "work").unwrap();
        set_tag_color(&conn, &work_tag.id, Some("#ff0055")).unwrap();
        add_tag_to_note(&conn, &note_b.id, "personal").unwrap();

        let all = list_all_note_tags(&conn).unwrap();
        assert_eq!(all.len(), 2);
        let a_assignment = all.iter().find(|a| a.note_id == note_a.id).unwrap();
        assert_eq!(a_assignment.color, Some("#ff0055".to_string()));
        let b_assignment = all.iter().find(|a| a.note_id == note_b.id).unwrap();
        assert_eq!(b_assignment.color, None);
    }
}
