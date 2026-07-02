use tauri::State;

use crate::bases;
use crate::models::{Base, BaseNoteAssignment};
use crate::state::{active_pool, AppState};

#[tauri::command]
pub fn list_bases(state: State<AppState>) -> Result<Vec<Base>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::list_bases(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_base(state: State<AppState>, name: String, color: String) -> Result<Base, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::create_base(&conn, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_base(
    state: State<AppState>,
    base_id: String,
    new_name: String,
) -> Result<Base, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::rename_base(&conn, &base_id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_base_color(
    state: State<AppState>,
    base_id: String,
    color: String,
) -> Result<Base, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::set_base_color(&conn, &base_id, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_base(state: State<AppState>, base_id: String) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::delete_base(&conn, &base_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_note_to_base(
    state: State<AppState>,
    base_id: String,
    note_id: String,
) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::add_note_to_base(&conn, &base_id, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_note_from_base(
    state: State<AppState>,
    base_id: String,
    note_id: String,
) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::remove_note_from_base(&conn, &base_id, &note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_all_base_notes(state: State<AppState>) -> Result<Vec<BaseNoteAssignment>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    bases::list_all_base_notes(&conn).map_err(|e| e.to_string())
}
