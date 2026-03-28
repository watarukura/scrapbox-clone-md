use crate::db::Database;
use crate::models::note::Note;
use crate::repositories::note_repository;

pub fn search_notes(db: &Database, workspace_id: &str, query: &str) -> Result<Vec<Note>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    note_repository::search_fts(&conn, workspace_id, query).map_err(|e| e.to_string())
}
