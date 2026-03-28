use crate::models::workspace::Workspace;
use rusqlite::{params, Connection};

pub fn insert_workspace(conn: &Connection, ws: &Workspace) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO workspaces (id, name, root_path, created_at, updated_at, last_scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(root_path) DO UPDATE SET
           name = excluded.name,
           updated_at = excluded.updated_at,
           last_scanned_at = excluded.last_scanned_at",
        params![
            ws.id,
            ws.name,
            ws.root_path,
            ws.created_at,
            ws.updated_at,
            ws.last_scanned_at,
        ],
    )?;
    Ok(())
}

pub fn get_workspace_by_path(
    conn: &Connection,
    root_path: &str,
) -> Result<Option<Workspace>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, root_path, created_at, updated_at, last_scanned_at FROM workspaces WHERE root_path = ?1")?;
    let mut rows = stmt.query_map(params![root_path], |row| {
        Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            root_path: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            last_scanned_at: row.get(5)?,
        })
    })?;
    match rows.next() {
        Some(Ok(ws)) => Ok(Some(ws)),
        Some(Err(e)) => Err(e),
        None => Ok(None),
    }
}

pub fn update_last_scanned(
    conn: &Connection,
    workspace_id: &str,
    scanned_at: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE workspaces SET last_scanned_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![scanned_at, workspace_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use std::path::PathBuf;

    fn setup() -> Connection {
        let db = Database::new(&PathBuf::from(":memory:")).unwrap();
        db.conn.into_inner().unwrap()
    }

    fn sample_workspace() -> Workspace {
        Workspace {
            id: "ws-1".to_string(),
            name: "My Workspace".to_string(),
            root_path: "/tmp/ws".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            last_scanned_at: None,
        }
    }

    #[test]
    fn insert_and_get_workspace() {
        let conn = setup();
        let ws = sample_workspace();
        insert_workspace(&conn, &ws).unwrap();

        let found = get_workspace_by_path(&conn, "/tmp/ws").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "ws-1");
        assert_eq!(found.name, "My Workspace");
    }

    #[test]
    fn get_workspace_returns_none_for_missing() {
        let conn = setup();
        let found = get_workspace_by_path(&conn, "/nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn insert_workspace_upserts_on_conflict() {
        let conn = setup();
        let mut ws = sample_workspace();
        insert_workspace(&conn, &ws).unwrap();

        ws.name = "Renamed".to_string();
        ws.updated_at = "2024-06-01T00:00:00Z".to_string();
        insert_workspace(&conn, &ws).unwrap();

        let found = get_workspace_by_path(&conn, "/tmp/ws").unwrap().unwrap();
        assert_eq!(found.name, "Renamed");
    }

    #[test]
    fn update_last_scanned_updates_fields() {
        let conn = setup();
        insert_workspace(&conn, &sample_workspace()).unwrap();

        update_last_scanned(&conn, "ws-1", "2024-12-01T00:00:00Z").unwrap();

        let found = get_workspace_by_path(&conn, "/tmp/ws").unwrap().unwrap();
        assert_eq!(found.last_scanned_at, Some("2024-12-01T00:00:00Z".to_string()));
        assert_eq!(found.updated_at, "2024-12-01T00:00:00Z");
    }
}
