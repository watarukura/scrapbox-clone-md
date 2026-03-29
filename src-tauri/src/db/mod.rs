use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let is_file_db = db_path.to_str() != Some(":memory:");

        match Self::try_open(db_path) {
            Ok(db) => Ok(db),
            Err(e) if is_file_db => {
                eprintln!(
                    "Database open failed ({e}). Removing corrupt database: {:?}",
                    db_path
                );
                let _ = std::fs::remove_file(db_path);
                let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
                let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
                Self::try_open(db_path)
            }
            Err(e) => Err(e),
        }
    }

    fn try_open(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        if !Self::check_integrity(&conn) {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CORRUPT),
                Some("integrity check failed".to_string()),
            ));
        }

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn check_integrity(conn: &Connection) -> bool {
        let main_ok = match conn.query_row("PRAGMA integrity_check", [], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(result) => result == "ok",
            Err(_) => false,
        };
        if !main_ok {
            return false;
        }

        // FTS5 tables can be corrupt even when main integrity_check passes
        Self::check_fts_integrity(conn)
    }

    fn check_fts_integrity(conn: &Connection) -> bool {
        // Check if note_fts table exists first
        let has_fts: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='note_fts'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_fts {
            return true;
        }

        match conn.execute(
            "INSERT INTO note_fts(note_fts) VALUES('integrity-check')",
            [],
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn rebuild_fts(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("DROP TABLE IF EXISTS note_fts;")?;
        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                note_id UNINDEXED,
                title,
                body_plaintext
            );",
        )?;
        conn.execute_batch(
            "INSERT INTO note_fts(note_id, title, body_plaintext)
             SELECT id, title, COALESCE(body_plaintext, '') FROM notes WHERE is_deleted = 0;",
        )?;
        Ok(())
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

        // Drop old content-sync FTS table if it exists (migration from old schema)
        {
            let has_old_fts: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='note_fts' AND sql LIKE '%content=%'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            if has_old_fts {
                conn.execute_batch("DROP TABLE IF EXISTS note_fts;")?;
            }
        }

        conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS note_fts USING fts5(
                note_id UNINDEXED,
                title,
                body_plaintext
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

    #[test]
    fn corrupt_db_is_recovered() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // Write garbage to simulate a corrupt database
        std::fs::write(&db_path, b"this is not a valid sqlite database").unwrap();

        // Database::new should recover by deleting and recreating
        let db = Database::new(&db_path).expect("Should recover from corrupt database");
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
    fn check_integrity_returns_true_for_valid_db() {
        let conn = Connection::open(":memory:").unwrap();
        assert!(Database::check_integrity(&conn));
    }

    #[test]
    fn corrupt_fts_db_is_recovered() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        // Create a valid database first
        {
            let db = Database::new(&db_path).expect("Should create database");
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "INSERT INTO workspaces (id, name, root_path, created_at, updated_at)
                 VALUES ('ws-1', 'test', '/tmp/test', '2024-01-01', '2024-01-01');
                 INSERT INTO notes (id, workspace_id, title, relative_path, absolute_path, file_name, body_markdown, indexed_at)
                 VALUES ('n-1', 'ws-1', 'Test', 'test.md', '/tmp/test/test.md', 'test.md', '# Test', '2024-01-01');
                 INSERT INTO note_fts(note_id, title, body_plaintext)
                 VALUES ('n-1', 'Test', '');",
            ).unwrap();
        }

        // Corrupt the FTS shadow tables by writing garbage into them
        {
            let conn = Connection::open(&db_path).unwrap();
            // Drop and recreate FTS data table with garbage to simulate corruption
            let _ = conn.execute_batch(
                "DELETE FROM note_fts_data;
                 INSERT INTO note_fts_data(id, block) VALUES(1, X'DEADBEEF');
                 INSERT INTO note_fts_data(id, block) VALUES(10, X'DEADBEEF');
                 INSERT INTO note_fts_data(id, block) VALUES(11, X'DEADBEEF');"
            );
        }

        // Database::new should detect FTS corruption and recover
        let db = Database::new(&db_path).expect("Should recover from FTS corruption");
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
}