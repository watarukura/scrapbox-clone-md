use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub relative_path: String,
    pub absolute_path: String,
    pub file_name: String,
    pub extension: String,
    pub frontmatter_json: Option<String>,
    pub body_markdown: String,
    pub body_plaintext: Option<String>,
    pub hash_sha256: Option<String>,
    pub file_created_at: Option<String>,
    pub file_modified_at: Option<String>,
    pub indexed_at: String,
    pub is_deleted: bool,
}
