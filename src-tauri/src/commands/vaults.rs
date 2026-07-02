use tauri::{AppHandle, Manager, State};

use crate::db::open_vault_pool;
use crate::models::VaultInfo;
use crate::state::{ActiveVault, AppState};
use crate::vault_registry;

fn app_data_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("could not resolve app data directory: {e}"))
}

#[tauri::command]
pub fn list_vaults(app: AppHandle) -> Result<Vec<VaultInfo>, String> {
    let dir = app_data_dir(&app)?;
    vault_registry::list_vaults(&dir).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn create_vault(
    app: AppHandle,
    state: State<AppState>,
    name: String,
    path: String,
) -> Result<VaultInfo, String> {
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        return Err("vault name cannot be empty".to_string());
    }

    std::fs::create_dir_all(&path).map_err(|e| format!("could not create vault directory: {e}"))?;

    let db_path = std::path::Path::new(&path).join("vault.sqlite");
    let pool = open_vault_pool(&db_path).map_err(|e| e.to_string())?;

    let data_dir = app_data_dir(&app)?;
    let info = vault_registry::register_vault(&data_dir, trimmed_name, &path)
        .map_err(|e| e.to_string())?;

    let mut guard = state
        .active
        .lock()
        .map_err(|_| "internal state lock was poisoned".to_string())?;
    *guard = Some(ActiveVault {
        id: info.id.clone(),
        path: path.clone(),
        pool,
    });

    Ok(info)
}

#[tauri::command]
pub fn open_vault(
    app: AppHandle,
    state: State<AppState>,
    vault_id: String,
) -> Result<VaultInfo, String> {
    let data_dir = app_data_dir(&app)?;
    let info = vault_registry::get_vault(&data_dir, &vault_id).map_err(|e| e.to_string())?;

    let db_path = std::path::Path::new(&info.path).join("vault.sqlite");
    let pool = open_vault_pool(&db_path).map_err(|e| e.to_string())?;

    let updated =
        vault_registry::touch_last_opened(&data_dir, &vault_id).map_err(|e| e.to_string())?;

    let mut guard = state
        .active
        .lock()
        .map_err(|_| "internal state lock was poisoned".to_string())?;
    *guard = Some(ActiveVault {
        id: updated.id.clone(),
        path: info.path.clone(),
        pool,
    });

    Ok(updated)
}

#[tauri::command]
pub fn close_vault(state: State<AppState>) -> Result<(), String> {
    let mut guard = state
        .active
        .lock()
        .map_err(|_| "internal state lock was poisoned".to_string())?;
    *guard = None;
    Ok(())
}

#[tauri::command]
pub fn current_vault(state: State<AppState>) -> Result<Option<String>, String> {
    let guard = state
        .active
        .lock()
        .map_err(|_| "internal state lock was poisoned".to_string())?;
    Ok(guard.as_ref().map(|v| v.id.clone()))
}
#[tauri::command]
pub fn delete_vault(
    app: AppHandle,
    state: State<AppState>,
    vault_id: String,
    delete_files: bool,
) -> Result<(), String> {
    let data_dir = app_data_dir(&app)?;
    let info = vault_registry::get_vault(&data_dir, &vault_id).map_err(|e| e.to_string())?;

    {
        let mut guard = state
            .active
            .lock()
            .map_err(|_| "internal state lock was poisoned".to_string())?;
        if guard.as_ref().map(|v| v.id.as_str()) == Some(vault_id.as_str()) {
            *guard = None;
        }
    }

    vault_registry::remove_vault(&data_dir, &vault_id).map_err(|e| e.to_string())?;

    if delete_files {
        let path = std::path::Path::new(&info.path);
        if path.exists() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("could not delete vault files: {e}"))?;
        }
    }

    Ok(())
}
