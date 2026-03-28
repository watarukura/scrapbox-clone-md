use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_scanned_at TEXT
            );",
        )?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                workspace_id TEXT NOT NULL,
                title TEXT NOT NULL,
                relative_path TEXT NOT NULL,
                absolute_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                extension TEXT NOT NULL DEFAULT 'md',
                frontmatter_json TEXT,
                body_markdown TEXT NOT NULL,
                body_plaintext TEXT,
                hash_sha256 TEXT,
                file_created_at TEXT,
                file_modified_at TEXT,
                indexed_at TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
                UNIQUE (workspace_id, relative_path)
            );",
        )?;

        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                title,
                body_plaintext,
                content='notes',
                content_rowid='rowid'
            );",
        )?;

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_notes_workspace_id ON notes(workspace_id);
             CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
             CREATE INDEX IF NOT EXISTS idx_notes_relative_path ON notes(relative_path);
             CREATE INDEX IF NOT EXISTS idx_notes_file_modified_at ON notes(file_modified_at);
             CREATE INDEX IF NOT EXISTS idx_notes_is_deleted ON notes(is_deleted);",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn in_memory_db() -> Database {
        let path = PathBuf::from(":memory:");
        Database::new(&path).expect("Failed to create in-memory database")
    }

    #[test]
    fn migrations_create_workspaces_table() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspaces'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrations_create_notes_table() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrations_create_fts_table() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='note_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrations_are_idempotent() {
        let db = in_memory_db();
        // Running migrations again should not fail
        db.run_migrations().expect("Second migration run should succeed");
    }

    #[test]
    fn migrations_create_indexes() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_notes_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 5);
    }
}