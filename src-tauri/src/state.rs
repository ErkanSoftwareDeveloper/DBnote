use std::sync::Mutex;

use crate::db::DbPool;
pub struct ActiveVault {
    pub id: String,
    pub path: String,
    pub pool: DbPool,
}

pub struct AppState {
    pub active: Mutex<Option<ActiveVault>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            active: Mutex::new(None),
        }
    }
}
pub fn active_pool(state: &AppState) -> Result<DbPool, String> {
    let guard = state
        .active
        .lock()
        .map_err(|_| "internal state lock was poisoned".to_string())?;
    match guard.as_ref() {
        Some(vault) => Ok(vault.pool.clone()),
        None => Err("no vault is currently open".to_string()),
    }
}
