use tauri::State;

use crate::folders;
use crate::models::Folder;
use crate::state::{active_pool, AppState};

#[tauri::command]
pub fn list_folders(state: State<AppState>) -> Result<Vec<Folder>, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    folders::list_folders(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_folder(
    state: State<AppState>,
    name: String,
    parent_id: Option<String>,
) -> Result<Folder, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    folders::create_folder(&conn, &name, parent_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_folder(
    state: State<AppState>,
    folder_id: String,
    new_name: String,
) -> Result<Folder, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    folders::rename_folder(&conn, &folder_id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_folder(
    state: State<AppState>,
    folder_id: String,
    new_parent_id: Option<String>,
) -> Result<Folder, String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    folders::move_folder(&conn, &folder_id, new_parent_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_folder(state: State<AppState>, folder_id: String) -> Result<(), String> {
    let pool = active_pool(&state)?;
    let conn = pool.get().map_err(|e| e.to_string())?;
    folders::delete_folder(&conn, &folder_id).map_err(|e| e.to_string())
}
