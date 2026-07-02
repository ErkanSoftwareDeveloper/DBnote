use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Base, BaseNoteAssignment};

const BASE_COLUMNS: &str = "b.id, b.name, b.color, COUNT(bn.note_id), b.created_at, b.updated_at";

fn row_to_base(row: &rusqlite::Row) -> rusqlite::Result<Base> {
    Ok(Base {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        note_count: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn get_base(conn: &Connection, base_id: &str) -> AppResult<Base> {
    conn.query_row(
        &format!(
            "SELECT {BASE_COLUMNS} FROM bases b
             LEFT JOIN base_notes bn ON bn.base_id = b.id
             WHERE b.id = ?1
             GROUP BY b.id"
        ),
        [base_id],
        row_to_base,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("base {base_id}")),
        other => AppError::from(other),
    })
}

pub fn list_bases(conn: &Connection) -> AppResult<Vec<Base>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BASE_COLUMNS} FROM bases b
         LEFT JOIN base_notes bn ON bn.base_id = b.id
         GROUP BY b.id
         ORDER BY b.name ASC"
    ))?;
    let rows = stmt.query_map([], row_to_base)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_base(conn: &Connection, name: &str, color: &str) -> AppResult<Base> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("base name cannot be empty".into()));
    }
    if !is_valid_hex_color(color) {
        return Err(AppError::Validation(format!(
            "'{color}' is not a valid hex color"
        )));
    }

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO bases (id, name, color) VALUES (?1, ?2, ?3)",
        (&id, trimmed, color),
    )?;
    get_base(conn, &id)
}

pub fn rename_base(conn: &Connection, base_id: &str, new_name: &str) -> AppResult<Base> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("base name cannot be empty".into()));
    }
    let updated = conn.execute(
        "UPDATE bases SET name = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (trimmed, base_id),
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("base {base_id}")));
    }
    get_base(conn, base_id)
}

pub fn set_base_color(conn: &Connection, base_id: &str, color: &str) -> AppResult<Base> {
    if !is_valid_hex_color(color) {
        return Err(AppError::Validation(format!(
            "'{color}' is not a valid hex color"
        )));
    }
    let updated = conn.execute(
        "UPDATE bases SET color = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?2",
        (color, base_id),
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(format!("base {base_id}")));
    }
    get_base(conn, base_id)
}

pub fn delete_base(conn: &Connection, base_id: &str) -> AppResult<()> {
    let exists: Option<String> = conn
        .query_row("SELECT id FROM bases WHERE id = ?1", [base_id], |r| {
            r.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("base {base_id}")));
    }
    conn.execute("DELETE FROM bases WHERE id = ?1", [base_id])?;
    Ok(())
}
pub fn add_note_to_base(conn: &Connection, base_id: &str, note_id: &str) -> AppResult<()> {
    let base_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM bases WHERE id = ?1)",
        [base_id],
        |r| r.get(0),
    )?;
    if !base_exists {
        return Err(AppError::NotFound(format!("base {base_id}")));
    }
    let note_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
        [note_id],
        |r| r.get(0),
    )?;
    if !note_exists {
        return Err(AppError::NotFound(format!("note {note_id}")));
    }

    conn.execute(
        "INSERT OR IGNORE INTO base_notes (base_id, note_id) VALUES (?1, ?2)",
        (base_id, note_id),
    )?;
    conn.execute(
        "UPDATE bases SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?1",
        [base_id],
    )?;
    Ok(())
}

pub fn remove_note_from_base(conn: &Connection, base_id: &str, note_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM base_notes WHERE base_id = ?1 AND note_id = ?2",
        (base_id, note_id),
    )?;
    Ok(())
}
pub fn list_all_base_notes(conn: &Connection) -> AppResult<Vec<BaseNoteAssignment>> {
    let mut stmt = conn.prepare(
        "SELECT bn.base_id, bn.note_id, b.color
         FROM base_notes bn
         JOIN bases b ON b.id = bn.base_id
         ORDER BY bn.created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(BaseNoteAssignment {
            base_id: row.get(0)?,
            note_id: row.get(1)?,
            base_color: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn is_valid_hex_color(s: &str) -> bool {
    let bytes = s.as_bytes();
    (bytes.len() == 4 || bytes.len() == 7)
        && bytes[0] == b'#'
        && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
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

    fn make_note(conn: &Connection, title: &str) -> String {
        create_note(
            conn,
            CreateNoteInput {
                title: title.to_string(),
                folder_id: None,
                content: None,
                content_format: None,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn creates_a_base_with_zero_notes() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        assert_eq!(base.name, "Work");
        assert_eq!(base.color, "#f97316");
        assert_eq!(base.note_count, 0);
    }

    #[test]
    fn rejects_blank_base_names() {
        let conn = setup();
        let result = create_base(&conn, "   ", "#f97316");
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn rejects_invalid_hex_colors() {
        let conn = setup();
        assert!(matches!(
            create_base(&conn, "Work", "orange"),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(
            create_base(&conn, "Work", "#zzzzzz"),
            Err(AppError::Validation(_))
        ));
        assert!(matches!(create_base(&conn, "Work", "#fff"), Ok(_)));
        assert!(matches!(create_base(&conn, "Work2", "#ffffff"), Ok(_)));
    }

    #[test]
    fn linking_notes_increases_note_count() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let note_a = make_note(&conn, "A");
        let note_b = make_note(&conn, "B");

        add_note_to_base(&conn, &base.id, &note_a).unwrap();
        let after_one = get_base(&conn, &base.id).unwrap();
        assert_eq!(after_one.note_count, 1);

        add_note_to_base(&conn, &base.id, &note_b).unwrap();
        let after_two = get_base(&conn, &base.id).unwrap();
        assert_eq!(after_two.note_count, 2);
    }

    #[test]
    fn linking_the_same_note_twice_is_idempotent() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let note = make_note(&conn, "A");

        add_note_to_base(&conn, &base.id, &note).unwrap();
        add_note_to_base(&conn, &base.id, &note).unwrap();

        let base_after = get_base(&conn, &base.id).unwrap();
        assert_eq!(base_after.note_count, 1);
    }

    #[test]
    fn linking_to_a_nonexistent_base_fails_clearly() {
        let conn = setup();
        let note = make_note(&conn, "A");
        let result = add_note_to_base(&conn, "does-not-exist", &note);
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn linking_a_nonexistent_note_fails_clearly() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let result = add_note_to_base(&conn, &base.id, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn removing_a_note_from_a_base_decreases_note_count() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let note = make_note(&conn, "A");
        add_note_to_base(&conn, &base.id, &note).unwrap();

        remove_note_from_base(&conn, &base.id, &note).unwrap();
        let after = get_base(&conn, &base.id).unwrap();
        assert_eq!(after.note_count, 0);
    }

    #[test]
    fn deleting_a_note_cascades_out_of_its_bases() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let note_id = make_note(&conn, "A");
        add_note_to_base(&conn, &base.id, &note_id).unwrap();

        conn.execute("DELETE FROM notes WHERE id = ?1", [&note_id])
            .unwrap();

        let after = get_base(&conn, &base.id).unwrap();
        assert_eq!(
            after.note_count, 0,
            "deleting a note must cascade out of base_notes"
        );
    }

    #[test]
    fn deleting_a_base_does_not_delete_its_notes() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();
        let note_id = make_note(&conn, "A");
        add_note_to_base(&conn, &base.id, &note_id).unwrap();

        delete_base(&conn, &base.id).unwrap();

        let still_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM notes WHERE id = ?1)",
                [&note_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(still_exists, "deleting a base must not delete its notes");
    }

    #[test]
    fn a_note_can_belong_to_multiple_bases() {
        let conn = setup();
        let work = create_base(&conn, "Work", "#f97316").unwrap();
        let project_x = create_base(&conn, "Project X", "#3b82f6").unwrap();
        let note = make_note(&conn, "Shared note");

        add_note_to_base(&conn, &work.id, &note).unwrap();
        add_note_to_base(&conn, &project_x.id, &note).unwrap();

        let assignments = list_all_base_notes(&conn).unwrap();
        let note_bases: Vec<&str> = assignments
            .iter()
            .filter(|a| a.note_id == note)
            .map(|a| a.base_id.as_str())
            .collect();
        assert_eq!(note_bases.len(), 2);
    }

    #[test]
    fn list_bases_orders_alphabetically_and_includes_counts() {
        let conn = setup();
        create_base(&conn, "Zebra", "#f97316").unwrap();
        let alpha = create_base(&conn, "Alpha", "#3b82f6").unwrap();
        let note = make_note(&conn, "A");
        add_note_to_base(&conn, &alpha.id, &note).unwrap();

        let bases = list_bases(&conn).unwrap();
        assert_eq!(bases.len(), 2);
        assert_eq!(bases[0].name, "Alpha");
        assert_eq!(bases[0].note_count, 1);
        assert_eq!(bases[1].name, "Zebra");
        assert_eq!(bases[1].note_count, 0);
    }

    #[test]
    fn renaming_and_recoloring_a_base() {
        let conn = setup();
        let base = create_base(&conn, "Work", "#f97316").unwrap();

        let renamed = rename_base(&conn, &base.id, "Career").unwrap();
        assert_eq!(renamed.name, "Career");

        let recolored = set_base_color(&conn, &base.id, "#22c55e").unwrap();
        assert_eq!(recolored.color, "#22c55e");
    }

    #[test]
    fn delete_base_fails_clearly_for_nonexistent_base() {
        let conn = setup();
        let result = delete_base(&conn, "does-not-exist");
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
