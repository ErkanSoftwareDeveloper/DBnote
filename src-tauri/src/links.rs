use rusqlite::Connection;
use uuid::Uuid;

use crate::error::AppResult;
use crate::link_parser::extract_wiki_links;
use crate::models::{Backlink, Link};

// Normalize Turkish dotted capital I before lowercase so title/link matching is stable.
fn fold(s: &str) -> String {
    s.chars()
        .map(|c| if c == '\u{0130}' { 'i' } else { c })
        .collect::<String>()
        .to_lowercase()
}

fn resolve_target(conn: &Connection, target_text: &str) -> AppResult<Option<String>> {
    let needle = fold(target_text.trim());
    let last_segment = fold(
        target_text
            .trim()
            .rsplit('/')
            .next()
            .unwrap_or(target_text.trim()),
    );

    // SQLite NOCASE is ASCII-only; compare in Rust so non-ASCII titles resolve.
    let mut stmt = conn.prepare("SELECT id, title, slug FROM notes")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let slug: String = row.get(2)?;
        if fold(&title) == needle
            || fold(&slug) == needle
            || fold(&title) == last_segment
            || fold(&slug) == last_segment
        {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

pub fn sync_links_for_note(conn: &Connection, note_id: &str, content: &str) -> AppResult<()> {
    conn.execute("DELETE FROM links WHERE source_note_id = ?1", [note_id])?;

    for link_ref in extract_wiki_links(content) {
        let target_id = resolve_target(conn, &link_ref.target)?;
        let is_broken = target_id.is_none();
        conn.execute(
            "INSERT INTO links (id, source_note_id, target_note_id, target_text, is_broken)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                Uuid::new_v4().to_string(),
                note_id,
                target_id,
                link_ref.target,
                is_broken,
            ),
        )?;
    }

    Ok(())
}

pub fn resolve_broken_links_targeting(
    conn: &Connection,
    note_id: &str,
    title: &str,
    slug: &str,
) -> AppResult<usize> {
    let title_folded = fold(title);
    let slug_folded = fold(slug);

    let mut stmt = conn.prepare(
        "SELECT id, target_text FROM links WHERE target_note_id IS NULL AND is_broken = 1",
    )?;
    let matching_ids: Vec<String> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .filter_map(|r| r.ok())
        .filter(|(_, text)| {
            let t = fold(text);
            t == title_folded || t == slug_folded
        })
        .map(|(id, _)| id)
        .collect();

    let mut updated = 0;
    for id in &matching_ids {
        updated += conn.execute(
            "UPDATE links SET target_note_id = ?1, is_broken = 0 WHERE id = ?2",
            (note_id, id),
        )?;
    }
    Ok(updated)
}

pub fn mark_links_broken_for_target(conn: &Connection, note_id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE links SET is_broken = 1 WHERE target_note_id = ?1",
        [note_id],
    )?;
    Ok(())
}

pub fn get_outgoing_links(conn: &Connection, note_id: &str) -> AppResult<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_note_id, target_note_id, target_text, is_broken, created_at
         FROM links WHERE source_note_id = ?1 ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([note_id], |row| {
        Ok(Link {
            id: row.get(0)?,
            source_note_id: row.get(1)?,
            target_note_id: row.get(2)?,
            target_text: row.get(3)?,
            is_broken: row.get::<_, i64>(4)? != 0,
            created_at: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn get_backlinks(conn: &Connection, note_id: &str) -> AppResult<Vec<Backlink>> {
    let mut stmt = conn.prepare(
        "SELECT l.source_note_id, n.title, n.slug, l.target_text
         FROM links l
         JOIN notes n ON n.id = l.source_note_id
         WHERE l.target_note_id = ?1
         ORDER BY n.title ASC",
    )?;
    let rows = stmt.query_map([note_id], |row| {
        Ok(Backlink {
            source_note_id: row.get(0)?,
            source_title: row.get(1)?,
            source_slug: row.get(2)?,
            target_text: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn list_all_links(conn: &Connection) -> AppResult<Vec<Link>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_note_id, target_note_id, target_text, is_broken, created_at
         FROM links ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Link {
            id: row.get(0)?,
            source_note_id: row.get(1)?,
            target_note_id: row.get(2)?,
            target_text: row.get(3)?,
            is_broken: row.get::<_, i64>(4)? != 0,
            created_at: row.get(5)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::run_migrations(&conn).unwrap();
        conn
    }

    fn insert_note(conn: &Connection, id: &str, title: &str, slug: &str, content: &str) {
        conn.execute(
            "INSERT INTO notes (id, title, slug, content) VALUES (?1, ?2, ?3, ?4)",
            (id, title, slug, content),
        )
        .unwrap();
    }

    #[test]
    fn sync_creates_broken_link_when_target_missing() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");

        sync_links_for_note(&conn, "n1", "See [[Nonexistent Note]] for more.").unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 1);
        assert!(links[0].is_broken);
        assert_eq!(links[0].target_note_id, None);
    }

    #[test]
    fn sync_resolves_link_to_existing_note() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "Target Note", "target-note", "");

        sync_links_for_note(&conn, "n1", "Link to [[Target Note]] here.").unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 1);
        assert!(!links[0].is_broken);
        assert_eq!(links[0].target_note_id, Some("n2".to_string()));
    }

    #[test]
    fn slug_target_disambiguates_duplicate_titles() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "Same", "same", "");
        insert_note(&conn, "n3", "Same", "same-2", "");

        sync_links_for_note(&conn, "n1", "[[same-2|Same]]").unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links[0].target_note_id, Some("n3".to_string()));
    }

    #[test]
    fn resolving_broken_links_after_target_created() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        sync_links_for_note(&conn, "n1", "Link to [[Future Note]] here.").unwrap();
        assert!(get_outgoing_links(&conn, "n1").unwrap()[0].is_broken);

        insert_note(&conn, "n2", "Future Note", "future-note", "");
        let updated =
            resolve_broken_links_targeting(&conn, "n2", "Future Note", "future-note").unwrap();
        assert_eq!(updated, 1);

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert!(!links[0].is_broken);
        assert_eq!(links[0].target_note_id, Some("n2".to_string()));
    }

    #[test]
    fn backlinks_are_returned_for_target_note() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "Target", "target", "");
        sync_links_for_note(&conn, "n1", "See [[Target]].").unwrap();

        let backlinks = get_backlinks(&conn, "n2").unwrap();
        assert_eq!(backlinks.len(), 1);
        assert_eq!(backlinks[0].source_note_id, "n1");
    }

    #[test]
    fn re_syncing_replaces_previous_links() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "A", "a", "");
        insert_note(&conn, "n3", "B", "b", "");

        sync_links_for_note(&conn, "n1", "[[A]]").unwrap();
        assert_eq!(get_outgoing_links(&conn, "n1").unwrap().len(), 1);

        sync_links_for_note(&conn, "n1", "[[A]] and [[B]]").unwrap();
        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn deleting_note_marks_incoming_links_broken() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "Target", "target", "");
        sync_links_for_note(&conn, "n1", "[[Target]]").unwrap();

        mark_links_broken_for_target(&conn, "n2").unwrap();
        conn.execute("DELETE FROM notes WHERE id = 'n2'", [])
            .unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 1);
        assert!(links[0].is_broken);
        assert_eq!(links[0].target_note_id, None);
    }

    #[test]
    fn list_all_links_returns_every_link_in_the_vault() {
        let conn = setup();
        insert_note(&conn, "n1", "A", "a", "");
        insert_note(&conn, "n2", "B", "b", "");
        insert_note(&conn, "n3", "C", "c", "");
        sync_links_for_note(&conn, "n1", "[[B]] and [[Unresolved]]").unwrap();
        sync_links_for_note(&conn, "n2", "[[C]]").unwrap();

        let all = list_all_links(&conn).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all.iter().filter(|l| l.is_broken).count(), 1);
    }

    #[test]
    fn turkish_characters_in_link_text_resolve_to_matching_note() {
        let conn = setup();
        insert_note(&conn, "n1", "Kaynak Not", "kaynak-not", "");
        insert_note(&conn, "n2", "Proje Planı", "proje-plani", "");

        sync_links_for_note(&conn, "n1", "Bugün [[Proje Planı]] üzerinde çalıştım.").unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 1);
        assert!(!links[0].is_broken, "[[Proje Planı]] must NOT be broken");
        assert_eq!(links[0].target_note_id.as_deref(), Some("n2"));
    }

    #[test]
    fn dotted_i_in_link_text_resolves_correctly() {
        let conn = setup();
        insert_note(&conn, "n1", "Source", "source", "");
        insert_note(&conn, "n2", "İş Notları", "is-notlari", "");

        sync_links_for_note(&conn, "n1", "[[İş Notları]] sayfasına bak.").unwrap();

        let links = get_outgoing_links(&conn, "n1").unwrap();
        assert_eq!(links.len(), 1);
        assert!(!links[0].is_broken, "[[İş Notları]] must NOT be broken");
    }
}
