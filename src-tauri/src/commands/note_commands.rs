use crate::db::Database;
use crate::models::note::Note;
use crate::services::note_service;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn list_notes(db: State<'_, Arc<Database>>, workspace_id: String) -> Result<Vec<Note>, String> {
    note_service::list_notes(&db, &workspace_id)
}

#[tauri::command]
pub fn get_note(db: State<'_, Arc<Database>>, note_id: String) -> Result<Option<Note>, String> {
    note_service::get_note(&db, &note_id)
}

#[tauri::command]
pub fn save_note(
    db: State<'_, Arc<Database>>,
    note_id: String,
    markdown: String,
) -> Result<Note, String> {
    note_service::save_note(&db, &note_id, &markdown)
}
