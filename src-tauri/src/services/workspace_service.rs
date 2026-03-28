use crate::db::Database;
use crate::models::workspace::Workspace;
use crate::repositories::workspace_repository;
use crate::services::index_service;
use chrono::Utc;
use uuid::Uuid;

pub fn open_workspace(db: &Database, root_path: &str) -> Result<Workspace, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if workspace already exists
    if let Some(ws) = workspace_repository::get_workspace_by_path(&conn, root_path)
        .map_err(|e| e.to_string())?
    {
        // Re-index
        drop(conn);
        index_service::full_scan(db, &ws)?;
        return Ok(ws);
    }

    let now = Utc::now().to_rfc3339();
    let name = std::path::Path::new(root_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());

    let ws = Workspace {
        id: Uuid::new_v4().to_string(),
        name,
        root_path: root_path.to_string(),
        created_at: now.clone(),
        updated_at: now,
        last_scanned_at: None,
    };

    workspace_repository::insert_workspace(&conn, &ws).map_err(|e| e.to_string())?;
    drop(conn);

    index_service::full_scan(db, &ws)?;

    Ok(ws)
}
