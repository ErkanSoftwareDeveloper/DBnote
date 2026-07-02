use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::VaultInfo;

const REGISTRY_FILE_NAME: &str = "vaults.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RegistryFile {
    vaults: Vec<VaultInfo>,
}

fn registry_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(REGISTRY_FILE_NAME)
}

fn load(app_data_dir: &Path) -> AppResult<RegistryFile> {
    let path = registry_path(app_data_dir);
    if !path.exists() {
        return Ok(RegistryFile::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(RegistryFile::default());
    }
    Ok(serde_json::from_str(&raw)?)
}

fn save(app_data_dir: &Path, registry: &RegistryFile) -> AppResult<()> {
    std::fs::create_dir_all(app_data_dir)?;
    let path = registry_path(app_data_dir);
    let raw = serde_json::to_string_pretty(registry)?;
    std::fs::write(path, raw)?;
    Ok(())
}

pub fn list_vaults(app_data_dir: &Path) -> AppResult<Vec<VaultInfo>> {
    Ok(load(app_data_dir)?.vaults)
}
pub fn register_vault(app_data_dir: &Path, name: &str, vault_path: &str) -> AppResult<VaultInfo> {
    let mut registry = load(app_data_dir)?;

    if registry.vaults.iter().any(|v| v.path == vault_path) {
        return Err(AppError::Conflict(format!(
            "a vault is already registered at {vault_path}"
        )));
    }

    let info = VaultInfo {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        path: vault_path.to_string(),
        created_at: now_iso(),
        last_opened_at: None,
    };

    registry.vaults.push(info.clone());
    save(app_data_dir, &registry)?;
    Ok(info)
}

pub fn get_vault(app_data_dir: &Path, vault_id: &str) -> AppResult<VaultInfo> {
    load(app_data_dir)?
        .vaults
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or_else(|| AppError::NotFound(format!("vault {vault_id}")))
}

pub fn touch_last_opened(app_data_dir: &Path, vault_id: &str) -> AppResult<VaultInfo> {
    let mut registry = load(app_data_dir)?;
    let vault = registry
        .vaults
        .iter_mut()
        .find(|v| v.id == vault_id)
        .ok_or_else(|| AppError::NotFound(format!("vault {vault_id}")))?;
    vault.last_opened_at = Some(now_iso());
    let updated = vault.clone();
    save(app_data_dir, &registry)?;
    Ok(updated)
}

pub fn remove_vault(app_data_dir: &Path, vault_id: &str) -> AppResult<VaultInfo> {
    let mut registry = load(app_data_dir)?;
    let position = registry
        .vaults
        .iter()
        .position(|v| v.id == vault_id)
        .ok_or_else(|| AppError::NotFound(format!("vault {vault_id}")))?;
    let removed = registry.vaults.remove(position);
    save(app_data_dir, &registry)?;
    Ok(removed)
}
fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let days = secs.div_euclid(86400);
    let secs_of_day = secs.rem_euclid(86400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn registers_and_lists_vaults() {
        let dir = tempdir().unwrap();
        let info = register_vault(dir.path(), "Personal", "/tmp/personal-vault").unwrap();
        assert_eq!(info.name, "Personal");

        let all = list_vaults(dir.path()).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, info.id);
    }

    #[test]
    fn rejects_duplicate_paths() {
        let dir = tempdir().unwrap();
        register_vault(dir.path(), "Personal", "/tmp/vault").unwrap();
        let result = register_vault(dir.path(), "Personal Again", "/tmp/vault");
        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[test]
    fn removing_a_vault_drops_it_from_the_registry() {
        let dir = tempdir().unwrap();
        let info = register_vault(dir.path(), "Personal", "/tmp/vault").unwrap();
        remove_vault(dir.path(), &info.id).unwrap();
        assert!(list_vaults(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn touching_last_opened_persists_across_loads() {
        let dir = tempdir().unwrap();
        let info = register_vault(dir.path(), "Personal", "/tmp/vault").unwrap();
        assert!(info.last_opened_at.is_none());

        touch_last_opened(dir.path(), &info.id).unwrap();
        let reloaded = get_vault(dir.path(), &info.id).unwrap();
        assert!(reloaded.last_opened_at.is_some());
    }

    #[test]
    fn civil_from_days_matches_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(1), (1970, 1, 2));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(civil_from_days(11016), (2000, 2, 29));
        assert_eq!(civil_from_days(11017), (2000, 3, 1));
        assert_eq!(civil_from_days(19357), (2022, 12, 31));
        assert_eq!(civil_from_days(19358), (2023, 1, 1));
    }

    #[test]
    fn now_iso_produces_a_format_javascripts_date_constructor_can_parse() {
        let ts = now_iso();
        assert_eq!(ts.len(), 20, "expected YYYY-MM-DDTHH:MM:SSZ, got: {ts}");
        assert!(
            ts.starts_with("20"),
            "expected a 2020s/2030s year, got: {ts}"
        );
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.as_bytes()[4], b'-');
        assert_eq!(ts.as_bytes()[7], b'-');
        assert_eq!(ts.as_bytes()[10], b'T');
        assert_eq!(ts.as_bytes()[13], b':');
        assert_eq!(ts.as_bytes()[16], b':');
    }

    #[test]
    fn registered_vault_has_a_real_iso_created_at() {
        let dir = tempdir().unwrap();
        let info = register_vault(dir.path(), "Personal", "/tmp/vault").unwrap();
        assert!(
            info.created_at.contains('T'),
            "created_at should be ISO-shaped: {}",
            info.created_at
        );
        assert!(info.created_at.ends_with('Z'));
    }
}
