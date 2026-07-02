use tauri::State;

use crate::models::{
    CreateNoteInput, Note, NoteSummary, NoteVersion, SearchHit, UpdateNoteContentInput,
};
use crate::notes;
use crate::state::{active_pool, AppState};

#[tauri::command]
pub fn create_note(state: State<AppState>, input: CreateNoteInput) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::create_note(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_note(state: State<AppState>, note_id: String) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::get_note(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note_content(
    state: State<AppState>,
    input: UpdateNoteContentInput,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::update_note_content(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_note(
    state: State<AppState>,
    note_id: String,
    new_title: String,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::rename_note(&conn, &note_id, &new_title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_note(
    state: State<AppState>,
    note_id: String,
    folder_id: Option<String>,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::move_note(&conn, &note_id, folder_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_pinned(
    state: State<AppState>,
    note_id: String,
    pinned: bool,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::set_pinned(&conn, &note_id, pinned).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_archived(
    state: State<AppState>,
    note_id: String,
    archived: bool,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::set_archived(&conn, &note_id, archived).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_properties(
    state: State<AppState>,
    note_id: String,
    properties: serde_json::Value,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::set_properties(&conn, &note_id, &properties).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn set_note_color(
    state: State<AppState>,
    note_id: String,
    color: Option<String>,
) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::set_color(&conn, &note_id, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(state: State<AppState>, note_id: String) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::delete_note(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes(
    state: State<AppState>,
    folder_id: Option<String>,
    include_archived: bool,
) -> Result<Vec<NoteSummary>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::list_notes(&conn, folder_id.as_deref(), include_archived).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_note_versions(
    state: State<AppState>,
    note_id: String,
) -> Result<Vec<NoteVersion>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::list_note_versions(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_note_version(state: State<AppState>, version_id: String) -> Result<Note, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::restore_note_version(&conn, &version_id).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn create_snapshot(
    state: State<AppState>,
    note_id: String,
) -> Result<crate::models::NoteVersion, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::create_snapshot(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_notes(
    state: State<AppState>,
    query: String,
    limit: i64,
) -> Result<Vec<SearchHit>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    notes::search_notes(&conn, &query, limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_backlinks(
    state: State<AppState>,
    note_id: String,
) -> Result<Vec<crate::models::Backlink>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::links::get_backlinks(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_outgoing_links(
    state: State<AppState>,
    note_id: String,
) -> Result<Vec<crate::models::Link>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::links::get_outgoing_links(&conn, &note_id).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn get_all_links(state: State<AppState>) -> Result<Vec<crate::models::Link>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    crate::links::list_all_links(&conn).map_err(|e| e.to_string())
}
