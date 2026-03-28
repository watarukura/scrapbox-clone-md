mod commands;
mod db;
mod fs;
mod markdown;
mod models;
mod repositories;
mod services;

use db::Database;
use notify::RecommendedWatcher;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct WatcherState(pub Arc<Mutex<Option<RecommendedWatcher>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("scrapbox-clone.db");
            let database = Arc::new(
                Database::new(&db_path).expect("Failed to initialize database"),
            );
            app.manage(database);
            app.manage(WatcherState(Arc::new(Mutex::new(None))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace_commands::open_workspace,
            commands::note_commands::list_notes,
            commands::note_commands::get_note,
            commands::note_commands::save_note,
            commands::search_commands::search_notes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
