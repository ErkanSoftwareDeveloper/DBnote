use std::path::Path;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

use crate::error::AppResult;

pub type DbPool = Pool<SqliteConnectionManager>;

// Append-only: existing vaults track these versions in `_migrations`.
const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "init", include_str!("../migrations/001_init.sql")),
    (
        2,
        "add_note_color",
        include_str!("../migrations/002_note_color.sql"),
    ),
    (3, "bases", include_str!("../migrations/003_bases.sql")),
];
pub fn open_vault_pool(path: &Path) -> AppResult<DbPool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let manager = SqliteConnectionManager::file(path).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA busy_timeout = 5000;",
        )?;
        Ok(())
    });

    let pool = Pool::builder().max_size(8).build(manager)?;

    {
        let conn = pool.get()?;
        run_migrations(&conn)?;
    }

    Ok(pool)
}
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         );",
    )?;

    let applied: i64 = conn.query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))?;

    if applied as usize >= MIGRATIONS.len() {
        return Ok(());
    }

    for (version, name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;
        if already_applied {
            continue;
        }

        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
            (version, name),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn opens_and_migrates_a_fresh_vault() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("vault.sqlite");
        let pool = open_vault_pool(&db_path).expect("pool should open and migrate cleanly");

        let conn = pool.get().unwrap();
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'notes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(table_count, 1, "notes table should exist after migration");
    }

    #[test]
    fn migrations_are_idempotent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("vault.sqlite");
        open_vault_pool(&db_path).unwrap();
        let pool = open_vault_pool(&db_path).unwrap();

        let conn = pool.get().unwrap();
        let applied: i64 = conn
            .query_row("SELECT COUNT(*) FROM _migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }
}
