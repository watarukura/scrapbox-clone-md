use crate::models::note::Note;
use rusqlite::{params, Connection};

pub fn upsert_note(conn: &Connection, note: &Note) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO notes (id, workspace_id, title, relative_path, absolute_path, file_name, extension,
         frontmatter_json, body_markdown, body_plaintext, hash_sha256, file_created_at, file_modified_at,
         indexed_at, is_deleted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(workspace_id, relative_path) DO UPDATE SET
           title = excluded.title,
           absolute_path = excluded.absolute_path,
           file_name = excluded.file_name,
           frontmatter_json = excluded.frontmatter_json,
           body_markdown = excluded.body_markdown,
           body_plaintext = excluded.body_plaintext,
           hash_sha256 = excluded.hash_sha256,
           file_created_at = excluded.file_created_at,
           file_modified_at = excluded.file_modified_at,
           indexed_at = excluded.indexed_at,
           is_deleted = excluded.is_deleted",
        params![
            note.id,
            note.workspace_id,
            note.title,
            note.relative_path,
            note.absolute_path,
            note.file_name,
            note.extension,
            note.frontmatter_json,
            note.body_markdown,
            note.body_plaintext,
            note.hash_sha256,
            note.file_created_at,
            note.file_modified_at,
            note.indexed_at,
            note.is_deleted as i32,
        ],
    )?;
    Ok(())
}

pub fn update_fts(conn: &Connection, note: &Note) -> Result<(), rusqlite::Error> {
    // Delete old entry if exists
    conn.execute(
        "DELETE FROM note_fts WHERE note_id = ?1",
        params![note.id],
    )?;
    // Insert new entry
    conn.execute(
        "INSERT INTO note_fts(note_id, title, body_plaintext) VALUES (?1, ?2, ?3)",
        params![note.id, note.title, note.body_plaintext.as_deref().unwrap_or("")],
    )?;
    Ok(())
}

pub fn list_notes_by_workspace(
    conn: &Connection,
    workspace_id: &str,
) -> Result<Vec<Note>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, title, relative_path, absolute_path, file_name, extension,
         frontmatter_json, body_markdown, body_plaintext, hash_sha256, file_created_at,
         file_modified_at, indexed_at, is_deleted
         FROM notes WHERE workspace_id = ?1 AND is_deleted = 0
         ORDER BY title ASC",
    )?;
    let rows = stmt.query_map(params![workspace_id], row_to_note)?;
    rows.collect()
}

pub fn get_note_by_id(conn: &Connection, note_id: &str) -> Result<Option<Note>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, title, relative_path, absolute_path, file_name, extension,
         frontmatter_json, body_markdown, body_plaintext, hash_sha256, file_created_at,
         file_modified_at, indexed_at, is_deleted
         FROM notes WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![note_id], row_to_note)?;
    match rows.next() {
        Some(Ok(n)) => Ok(Some(n)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

pub fn get_note_by_path(
    conn: &Connection,
    workspace_id: &str,
    relative_path: &str,
) -> Result<Option<Note>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, title, relative_path, absolute_path, file_name, extension,
         frontmatter_json, body_markdown, body_plaintext, hash_sha256, file_created_at,
         file_modified_at, indexed_at, is_deleted
         FROM notes WHERE workspace_id = ?1 AND relative_path = ?2",
    )?;
    let mut rows = stmt.query_map(params![workspace_id, relative_path], row_to_note)?;
    match rows.next() {
        Some(Ok(n)) => Ok(Some(n)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

pub fn mark_deleted(
    conn: &Connection,
    workspace_id: &str,
    relative_path: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE notes SET is_deleted = 1 WHERE workspace_id = ?1 AND relative_path = ?2",
        params![workspace_id, relative_path],
    )?;
    Ok(())
}

pub fn get_all_relative_paths(
    conn: &Connection,
    workspace_id: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT relative_path FROM notes WHERE workspace_id = ?1 AND is_deleted = 0",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| row.get(0))?;
    rows.collect()
}

pub fn search_fts(
    conn: &Connection,
    workspace_id: &str,
    query: &str,
) -> Result<Vec<Note>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT n.id, n.workspace_id, n.title, n.relative_path, n.absolute_path, n.file_name,
         n.extension, n.frontmatter_json, n.body_markdown, n.body_plaintext, n.hash_sha256,
         n.file_created_at, n.file_modified_at, n.indexed_at, n.is_deleted
         FROM notes n
         JOIN note_fts f ON f.note_id = n.id
         WHERE note_fts MATCH ?1 AND n.workspace_id = ?2 AND n.is_deleted = 0
         ORDER BY rank",
    )?;
    let rows = stmt.query_map(params![query, workspace_id], row_to_note)?;
    rows.collect()
}

fn row_to_note(row: &rusqlite::Row) -> Result<Note, rusqlite::Error> {
    let is_deleted_int: i32 = row.get(14)?;
    Ok(Note {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        title: row.get(2)?,
        relative_path: row.get(3)?,
        absolute_path: row.get(4)?,
        file_name: row.get(5)?,
        extension: row.get(6)?,
        frontmatter_json: row.get(7)?,
        body_markdown: row.get(8)?,
        body_plaintext: row.get(9)?,
        hash_sha256: row.get(10)?,
        file_created_at: row.get(11)?,
        file_modified_at: row.get(12)?,
        indexed_at: row.get(13)?,
        is_deleted: is_deleted_int != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::workspace::Workspace;
    use crate::repositories::workspace_repository;
    use std::path::PathBuf;

    fn setup() -> Connection {
        let db = Database::new(&PathBuf::from(":memory:")).unwrap();
        let conn = db.conn.into_inner().unwrap();
        conn
    }

    fn sample_workspace() -> Workspace {
        Workspace {
            id: "ws-1".to_string(),
            name: "test".to_string(),
            root_path: "/tmp/test".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            last_scanned_at: None,
        }
    }

    fn sample_note(id: &str, relative_path: &str) -> Note {
        Note {
            id: id.to_string(),
            workspace_id: "ws-1".to_string(),
            title: "Test Note".to_string(),
            relative_path: relative_path.to_string(),
            absolute_path: format!("/tmp/test/{}", relative_path),
            file_name: relative_path.to_string(),
            extension: "md".to_string(),
            frontmatter_json: None,
            body_markdown: "# Test\nHello".to_string(),
            body_plaintext: Some("Test Hello".to_string()),
            hash_sha256: Some("abc123".to_string()),
            file_created_at: Some("2024-01-01T00:00:00Z".to_string()),
            file_modified_at: Some("2024-01-01T00:00:00Z".to_string()),
            indexed_at: "2024-01-01T00:00:00Z".to_string(),
            is_deleted: false,
        }
    }

    fn setup_with_workspace() -> Connection {
        let conn = setup();
        workspace_repository::insert_workspace(&conn, &sample_workspace()).unwrap();
        conn
    }

    #[test]
    fn upsert_and_get_by_id() {
        let conn = setup_with_workspace();
        let note = sample_note("n-1", "hello.md");
        upsert_note(&conn, &note).unwrap();

        let found = get_note_by_id(&conn, "n-1").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.title, "Test Note");
        assert_eq!(found.is_deleted, false);
    }

    #[test]
    fn upsert_updates_existing_note() {
        let conn = setup_with_workspace();
        let mut note = sample_note("n-1", "hello.md");
        upsert_note(&conn, &note).unwrap();

        note.title = "Updated Title".to_string();
        upsert_note(&conn, &note).unwrap();

        let found = get_note_by_id(&conn, "n-1").unwrap().unwrap();
        assert_eq!(found.title, "Updated Title");
    }

    #[test]
    fn get_note_by_id_returns_none_for_missing() {
        let conn = setup_with_workspace();
        let found = get_note_by_id(&conn, "nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn get_note_by_path_found() {
        let conn = setup_with_workspace();
        upsert_note(&conn, &sample_note("n-1", "hello.md")).unwrap();

        let found = get_note_by_path(&conn, "ws-1", "hello.md").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "n-1");
    }

    #[test]
    fn get_note_by_path_not_found() {
        let conn = setup_with_workspace();
        let found = get_note_by_path(&conn, "ws-1", "missing.md").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn list_notes_excludes_deleted() {
        let conn = setup_with_workspace();
        upsert_note(&conn, &sample_note("n-1", "a.md")).unwrap();
        upsert_note(&conn, &sample_note("n-2", "b.md")).unwrap();
        mark_deleted(&conn, "ws-1", "b.md").unwrap();

        let notes = list_notes_by_workspace(&conn, "ws-1").unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, "n-1");
    }

    #[test]
    fn mark_deleted_sets_flag() {
        let conn = setup_with_workspace();
        upsert_note(&conn, &sample_note("n-1", "hello.md")).unwrap();
        mark_deleted(&conn, "ws-1", "hello.md").unwrap();

        let found = get_note_by_id(&conn, "n-1").unwrap().unwrap();
        assert!(found.is_deleted);
    }

    #[test]
    fn get_all_relative_paths_excludes_deleted() {
        let conn = setup_with_workspace();
        upsert_note(&conn, &sample_note("n-1", "a.md")).unwrap();
        upsert_note(&conn, &sample_note("n-2", "b.md")).unwrap();
        mark_deleted(&conn, "ws-1", "a.md").unwrap();

        let paths = get_all_relative_paths(&conn, "ws-1").unwrap();
        assert_eq!(paths, vec!["b.md"]);
    }

    fn rebuild_fts(conn: &Connection) {
        conn.execute_batch("DELETE FROM note_fts;").unwrap();
        conn.execute_batch(
            "INSERT INTO note_fts(note_id, title, body_plaintext)
             SELECT id, title, COALESCE(body_plaintext, '') FROM notes WHERE is_deleted = 0;"
        ).unwrap();
    }

    #[test]
    fn fts_search_finds_matching_note() {
        let conn = setup_with_workspace();
        let note = sample_note("n-1", "hello.md");
        upsert_note(&conn, &note).unwrap();
        rebuild_fts(&conn);

        let results = search_fts(&conn, "ws-1", "Test").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "n-1");
    }

    #[test]
    fn fts_search_no_match() {
        let conn = setup_with_workspace();
        let note = sample_note("n-1", "hello.md");
        upsert_note(&conn, &note).unwrap();
        rebuild_fts(&conn);

        let results = search_fts(&conn, "ws-1", "zzzznotfound").unwrap();
        assert_eq!(results.len(), 0);
    }
}
