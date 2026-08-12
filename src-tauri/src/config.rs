use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub favorites: Vec<u16>,
    pub collapsed: bool,
    pub poll_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            favorites: vec![8080],
            collapsed: false,
            poll_interval_ms: 2000,
        }
    }
}

/// PRD §9 names this location literally, so it is built from APPDATA rather than
/// Tauri's app_config_dir (which would use the bundle identifier instead).
fn config_path() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA is not set".to_string())?;
    Ok(PathBuf::from(appdata).join("portpeek").join("config.json"))
}

/// Infallible by design: a missing, unreadable or corrupt config must never stop
/// the widget from starting. Anything unusable degrades to defaults, which is also
/// how the default 8080 favourite gets seeded on first run.
#[tauri::command]
pub fn get_config() -> Config {
    config_path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|raw| parse(&raw))
        .map(sanitize)
        .unwrap_or_default()
}

/// PRD §9 documents this file as user-editable, and on Windows the obvious editors
/// (Notepad, PowerShell's `Set-Content -Encoding utf8`) prepend a UTF-8 BOM.
/// serde_json rejects a leading BOM, which would silently reset the user's
/// favourites to defaults — so strip it rather than treat the file as corrupt.
fn parse(raw: &str) -> Option<Config> {
    serde_json::from_str(raw.trim_start_matches('\u{feff}')).ok()
}

#[tauri::command]
pub fn save_config(config: Config) -> Result<Config, String> {
    let path = config_path()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    let clean = sanitize(config);
    let json = serde_json::to_string_pretty(&clean).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    Ok(clean)
}

/// Port 0 is not a real listener and duplicates would render as duplicate chips,
/// so both are dropped on the way in AND on the way out — a hand-edited file gets
/// the same treatment as UI input.
fn sanitize(mut config: Config) -> Config {
    config.favorites.retain(|&p| p != 0);
    config.favorites.sort_unstable();
    config.favorites.dedup();
    if config.poll_interval_ms < 250 {
        config.poll_interval_ms = Config::default().poll_interval_ms;
    }
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrupt_config_degrades_to_defaults_rather_than_failing() {
        assert!(serde_json::from_str::<Config>("{ not json").is_err());
        // get_config swallows that error path; the default seeds 8080 (PRD §5).
        assert_eq!(Config::default().favorites, vec![8080]);
    }

    /// Regression: this shipped broken. Seeding the real file from PowerShell wrote
    /// a BOM, get_config silently fell back to defaults, and the seeded favourites
    /// vanished. The earlier tests missed it because Rust literals have no BOM.
    #[test]
    fn a_bom_does_not_silently_reset_favorites() {
        let raw = "\u{feff}{ \"favorites\": [9200, 8080], \"collapsed\": false, \"pollIntervalMs\": 2000 }";
        let parsed = parse(raw).expect("a BOM-prefixed config must still parse");
        assert_eq!(sanitize(parsed).favorites, vec![8080, 9200]);

        // Plain UTF-8 must keep working, and genuine garbage must still be rejected.
        assert!(parse(r#"{"favorites":[80],"collapsed":false,"pollIntervalMs":2000}"#).is_some());
        assert!(parse("{ not json").is_none());
    }

    #[test]
    fn favorites_are_sorted_deduped_and_zero_stripped() {
        let messy = Config {
            favorites: vec![8080, 3000, 8080, 0, 1420],
            collapsed: false,
            poll_interval_ms: 2000,
        };
        assert_eq!(sanitize(messy).favorites, vec![1420, 3000, 8080]);
    }

    #[test]
    fn a_hand_edited_poll_interval_cannot_spin_the_cpu() {
        let hot = Config {
            favorites: vec![],
            collapsed: false,
            poll_interval_ms: 1,
        };
        assert_eq!(sanitize(hot).poll_interval_ms, 2000);
    }
}
