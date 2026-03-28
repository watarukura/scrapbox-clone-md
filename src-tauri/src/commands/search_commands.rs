use crate::db::Database;
use crate::models::note::Note;
use crate::services::search_service;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub fn search_notes(
    db: State<'_, Arc<Database>>,
    workspace_id: String,
    query: String,
) -> Result<Vec<Note>, String> {
    search_service::search_notes(&db, &workspace_id, &query)
}
