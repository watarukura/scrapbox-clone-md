use crate::db::Database;
use crate::fs::writer;
use crate::markdown::frontmatter::{extract_title, parse_frontmatter};
use crate::markdown::plaintext::markdown_to_plaintext;
use crate::models::note::Note;
use crate::repositories::note_repository;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn get_note(db: &Database, note_id: &str) -> Result<Option<Note>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    note_repository::get_note_by_id(&conn, note_id).map_err(|e| e.to_string())
}

pub fn list_notes(db: &Database, workspace_id: &str) -> Result<Vec<Note>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    note_repository::list_notes_by_workspace(&conn, workspace_id).map_err(|e| e.to_string())
}

pub fn save_note(db: &Database, note_id: &str, markdown: &str) -> Result<Note, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut note = note_repository::get_note_by_id(&conn, note_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note not found: {}", note_id))?;

    // 1. Write file
    let abs_path = Path::new(&note.absolute_path);
    writer::write_file_atomic(abs_path, markdown).map_err(|e| e.to_string())?;

    // 2. Recompute hash
    let hash = hex::encode(Sha256::digest(markdown.as_bytes()));

    // 3. Parse
    let parsed = parse_frontmatter(markdown);
    let plaintext = markdown_to_plaintext(&parsed.body);
    let title = extract_title(&parsed.frontmatter_json, &parsed.body, &note.file_name);

    // 4. Update note
    let now = Utc::now().to_rfc3339();
    note.title = title;
    note.body_markdown = markdown.to_string();
    note.body_plaintext = Some(plaintext);
    note.frontmatter_json = parsed.frontmatter_json;
    note.hash_sha256 = Some(hash);
    note.file_modified_at = Some(now.clone());
    note.indexed_at = now;
    note.is_deleted = false;

    // 5. Update DB + FTS
    note_repository::upsert_note(&conn, &note).map_err(|e| e.to_string())?;
    note_repository::update_fts(&conn, &note).map_err(|e| e.to_string())?;

    Ok(note)
}
