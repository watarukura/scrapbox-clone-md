use crate::db::Database;
use crate::fs::reader;
use crate::fs::scanner;
use crate::markdown::frontmatter::{extract_title, parse_frontmatter};
use crate::markdown::plaintext::markdown_to_plaintext;
use crate::models::note::Note;
use crate::models::workspace::Workspace;
use crate::repositories::{note_repository, workspace_repository};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

pub fn full_scan(db: &Database, workspace: &Workspace) -> Result<(), String> {
    match full_scan_inner(db, workspace) {
        Ok(()) => Ok(()),
        Err(e) if e.contains("malformed") || e.contains("corrupt") || e.contains("fts5") => {
            eprintln!("FTS corruption detected during full_scan: {e}. Rebuilding FTS index...");
            db.rebuild_fts().map_err(|e| e.to_string())?;
            full_scan_inner(db, workspace)
        }
        Err(e) => Err(e),
    }
}

fn full_scan_inner(db: &Database, workspace: &Workspace) -> Result<(), String> {
    let root = Path::new(&workspace.root_path);
    let files = scanner::scan_md_files(root);
    let now = Utc::now().to_rfc3339();

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut scanned_paths: HashSet<String> = HashSet::new();

    for file_path in &files {
        let relative = file_path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        scanned_paths.insert(relative.clone());

        let content = reader::read_file(file_path).map_err(|e| e.to_string())?;
        let hash = hex::encode(Sha256::digest(content.as_bytes()));

        // Check if note exists and hash matches
        if let Some(existing) =
            note_repository::get_note_by_path(&conn, &workspace.id, &relative)
                .map_err(|e| e.to_string())?
        {
            if existing.hash_sha256.as_deref() == Some(&hash) {
                continue;
            }
        }

        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let parsed = parse_frontmatter(&content);
        let plaintext = markdown_to_plaintext(&parsed.body);
        let title = extract_title(&parsed.frontmatter_json, &parsed.body, &file_name);

        let metadata = std::fs::metadata(file_path).ok();
        let file_created_at = metadata
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339());
        let file_modified_at = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339());

        let existing_id =
            note_repository::get_note_by_path(&conn, &workspace.id, &relative)
                .map_err(|e| e.to_string())?
                .map(|n| n.id);

        let note = Note {
            id: existing_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            workspace_id: workspace.id.clone(),
            title,
            relative_path: relative,
            absolute_path: file_path.to_string_lossy().to_string(),
            file_name,
            extension: "md".to_string(),
            frontmatter_json: parsed.frontmatter_json,
            body_markdown: content,
            body_plaintext: Some(plaintext),
            hash_sha256: Some(hash),
            file_created_at,
            file_modified_at,
            indexed_at: now.clone(),
            is_deleted: false,
        };

        note_repository::upsert_note(&conn, &note).map_err(|e| e.to_string())?;
        note_repository::update_fts(&conn, &note).map_err(|e| e.to_string())?;
    }

    // Mark deleted files
    let db_paths =
        note_repository::get_all_relative_paths(&conn, &workspace.id).map_err(|e| e.to_string())?;
    for path in db_paths {
        if !scanned_paths.contains(&path) {
            note_repository::mark_deleted(&conn, &workspace.id, &path)
                .map_err(|e| e.to_string())?;
        }
    }

    workspace_repository::update_last_scanned(&conn, &workspace.id, &now)
        .map_err(|e| e.to_string())?;

    Ok(())
}
