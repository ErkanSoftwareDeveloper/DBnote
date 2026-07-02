use tauri::State;

use crate::models::Tag;
use crate::state::{active_pool, AppState};
use crate::tags;

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Result<Vec<Tag>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::list_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags_for_note(state: State<AppState>, note_id: String) -> Result<Vec<Tag>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::list_tags_for_note(&conn, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_notes_for_tag(state: State<AppState>, tag_id: String) -> Result<Vec<String>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::list_notes_for_tag(&conn, &tag_id).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn list_all_note_tags(
    state: State<AppState>,
) -> Result<Vec<crate::models::NoteTagAssignment>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::list_all_note_tags(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_tag_to_note(
    state: State<AppState>,
    note_id: String,
    tag_name: String,
) -> Result<Tag, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::add_tag_to_note(&conn, &note_id, &tag_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_tag_from_note(
    state: State<AppState>,
    note_id: String,
    tag_id: String,
) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::remove_tag_from_note(&conn, &note_id, &tag_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_tag_color(
    state: State<AppState>,
    tag_id: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    tags::set_tag_color(&conn, &tag_id, color.as_deref()).map_err(|e| e.to_string())
}
