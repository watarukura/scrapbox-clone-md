use crate::db::Database;
use crate::models::workspace::Workspace;
use crate::services::{file_watch_service, workspace_service};
use crate::WatcherState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn open_workspace(
    app_handle: tauri::AppHandle,
    db: State<'_, Arc<Database>>,
    watcher_state: State<'_, WatcherState>,
    root_path: String,
) -> Result<Workspace, String> {
    let db = Arc::clone(&db);
    let watcher_state_inner = watcher_state.0.clone();

    let db_clone = Arc::clone(&db);
    let ws = tauri::async_runtime::spawn_blocking(move || {
        workspace_service::open_workspace(&db_clone, &root_path)
    })
    .await
    .map_err(|e| e.to_string())?
    ?;

    // Start file watching
    let watcher = file_watch_service::start_watching(
        app_handle,
        db,
        ws.clone(),
    )?;
    let mut guard = watcher_state_inner.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    *guard = Some(watcher);

    Ok(ws)
}
