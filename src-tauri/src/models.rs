use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub path: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub content_format: String,
    pub properties: serde_json::Value,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub word_count: i64,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: String,
    pub folder_id: Option<String>,
    pub title: String,
    pub slug: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub content_preview: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteVersion {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTagAssignment {
    pub note_id: String,
    pub tag_id: String,
    pub tag_name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub id: String,
    pub source_note_id: String,
    pub target_note_id: Option<String>,
    pub target_text: String,
    pub is_broken: bool,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backlink {
    pub source_note_id: String,
    pub source_title: String,
    pub source_slug: String,
    pub target_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub note_id: String,
    pub title: String,
    pub slug: String,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub last_opened_at: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base {
    pub id: String,
    pub name: String,
    pub color: String,
    pub note_count: i64,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseNoteAssignment {
    pub base_id: String,
    pub note_id: String,
    pub base_color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteInput {
    pub title: String,
    pub folder_id: Option<String>,
    pub content: Option<String>,
    pub content_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateNoteContentInput {
    pub note_id: String,
    pub content: String,
    pub snapshot_previous: bool,
}
