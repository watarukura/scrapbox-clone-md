use crate::db::Database;
use crate::services::index_service;
use crate::models::workspace::Workspace;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

pub fn start_watching(
    app_handle: AppHandle,
    db: Arc<Database>,
    workspace: Workspace,
) -> Result<RecommendedWatcher, String> {
    let root_path = workspace.root_path.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        let is_md = event.paths.iter().any(|p| {
                            p.extension().map_or(false, |ext| ext == "md")
                        });
                        if is_md {
                            if let Err(e) = index_service::full_scan(&db, &workspace) {
                                eprintln!("Re-index failed: {}", e);
                            }
                            let _ = app_handle.emit("notes-changed", ());
                        }
                    }
                    _ => {}
                }
            }
        },
        Config::default(),
    )
    .map_err(|e| e.to_string())?;

    watcher
        .watch(Path::new(&root_path), RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    Ok(watcher)
}
