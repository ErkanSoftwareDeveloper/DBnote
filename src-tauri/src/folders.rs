use rusqlite::{Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Folder;

fn row_to_folder(row: &rusqlite::Row) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        path: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

const SELECT_COLUMNS: &str = "id, parent_id, name, path, sort_order, created_at, updated_at";

pub fn get_folder(conn: &Connection, folder_id: &str) -> AppResult<Folder> {
    conn.query_row(
        &format!("SELECT {SELECT_COLUMNS} FROM folders WHERE id = ?1"),
        [folder_id],
        row_to_folder,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("folder {folder_id}")),
        other => AppError::from(other),
    })
}

pub fn list_folders(conn: &Connection) -> AppResult<Vec<Folder>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLUMNS} FROM folders ORDER BY parent_id IS NOT NULL, sort_order ASC, name ASC"
    ))?;
    let rows = stmt.query_map([], row_to_folder)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn create_folder(conn: &Connection, name: &str, parent_id: Option<&str>) -> AppResult<Folder> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("folder name cannot be empty".into()));
    }

    let path = match parent_id {
        Some(pid) => {
            let parent = get_folder(conn, pid)?;
            format!("{}/{}", parent.path, trimmed)
        }
        None => trimmed.to_string(),
    };

    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO folders (id, parent_id, name, path) VALUES (?1, ?2, ?3, ?4)",
        (&id, parent_id, trimmed, &path),
    )?;

    get_folder(conn, &id)
}
pub fn rename_folder(conn: &Connection, folder_id: &str, new_name: &str) -> AppResult<Folder> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("folder name cannot be empty".into()));
    }

    let folder = get_folder(conn, folder_id)?;
    let new_path = match folder.path.rfind('/') {
        Some(idx) => format!("{}/{}", &folder.path[..idx], trimmed),
        None => trimmed.to_string(),
    };

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE folders SET name = ?1, path = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?3",
        (trimmed, &new_path, folder_id),
    )?;
    tx.execute(
        "UPDATE folders
         SET path = ?1 || substr(path, ?2)
         WHERE path LIKE ?3",
        (
            &new_path,
            folder.path.len() as i64 + 1,
            format!("{}/%", folder.path),
        ),
    )?;
    tx.commit()?;

    get_folder(conn, folder_id)
}
pub fn move_folder(
    conn: &Connection,
    folder_id: &str,
    new_parent_id: Option<&str>,
) -> AppResult<Folder> {
    if let Some(new_parent_id) = new_parent_id {
        if new_parent_id == folder_id {
            return Err(AppError::Validation(
                "a folder cannot be moved into itself".into(),
            ));
        }
        let descendant_paths = list_folders(conn)?;
        let folder = get_folder(conn, folder_id)?;
        let would_cycle = descendant_paths
            .iter()
            .any(|f| f.id == new_parent_id && f.path.starts_with(&format!("{}/", folder.path)));
        if would_cycle {
            return Err(AppError::Validation(
                "a folder cannot be moved into one of its own descendants".into(),
            ));
        }
    }

    let folder = get_folder(conn, folder_id)?;
    let new_path = match new_parent_id {
        Some(pid) => {
            let parent = get_folder(conn, pid)?;
            format!("{}/{}", parent.path, folder.name)
        }
        None => folder.name.clone(),
    };

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "UPDATE folders SET parent_id = ?1, path = ?2, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?3",
        (new_parent_id, &new_path, folder_id),
    )?;
    tx.execute(
        "UPDATE folders
         SET path = ?1 || substr(path, ?2)
         WHERE path LIKE ?3",
        (
            &new_path,
            folder.path.len() as i64 + 1,
            format!("{}/%", folder.path),
        ),
    )?;
    tx.commit()?;

    get_folder(conn, folder_id)
}

pub fn delete_folder(conn: &Connection, folder_id: &str) -> AppResult<()> {
    let exists: Option<String> = conn
        .query_row("SELECT id FROM folders WHERE id = ?1", [folder_id], |r| {
            r.get(0)
        })
        .optional()?;
    if exists.is_none() {
        return Err(AppError::NotFound(format!("folder {folder_id}")));
    }
    conn.execute("DELETE FROM folders WHERE id = ?1", [folder_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn creates_root_and_nested_folders_with_correct_paths() {
        let conn = setup();
        let root = create_folder(&conn, "Projects", None).unwrap();
        assert_eq!(root.path, "Projects");

        let child = create_folder(&conn, "Research", Some(&root.id)).unwrap();
        assert_eq!(child.path, "Projects/Research");
    }

    #[test]
    fn rejects_blank_folder_names() {
        let conn = setup();
        let result = create_folder(&conn, "   ", None);
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn renaming_a_folder_cascades_to_descendants() {
        let conn = setup();
        let root = create_folder(&conn, "Projects", None).unwrap();
        let child = create_folder(&conn, "Research", Some(&root.id)).unwrap();
        let grandchild = create_folder(&conn, "Notes", Some(&child.id)).unwrap();

        rename_folder(&conn, &root.id, "Work").unwrap();

        let child_after = get_folder(&conn, &child.id).unwrap();
        let grandchild_after = get_folder(&conn, &grandchild.id).unwrap();
        assert_eq!(child_after.path, "Work/Research");
        assert_eq!(grandchild_after.path, "Work/Research/Notes");
    }

    #[test]
    fn moving_a_folder_updates_its_subtree() {
        let conn = setup();
        let a = create_folder(&conn, "A", None).unwrap();
        let b = create_folder(&conn, "B", None).unwrap();
        let child = create_folder(&conn, "Child", Some(&a.id)).unwrap();

        move_folder(&conn, &child.id, Some(&b.id)).unwrap();

        let moved = get_folder(&conn, &child.id).unwrap();
        assert_eq!(moved.path, "B/Child");
        assert_eq!(moved.parent_id, Some(b.id));
    }

    #[test]
    fn moving_a_folder_into_its_own_descendant_is_rejected() {
        let conn = setup();
        let parent = create_folder(&conn, "Parent", None).unwrap();
        let child = create_folder(&conn, "Child", Some(&parent.id)).unwrap();

        let result = move_folder(&conn, &parent.id, Some(&child.id));
        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[test]
    fn deleting_a_folder_cascades_to_children() {
        let conn = setup();
        let parent = create_folder(&conn, "Parent", None).unwrap();
        let child = create_folder(&conn, "Child", Some(&parent.id)).unwrap();

        delete_folder(&conn, &parent.id).unwrap();

        let result = get_folder(&conn, &child.id);
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
