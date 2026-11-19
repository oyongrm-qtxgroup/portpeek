use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";

/// Passed by the autostart registration so a sign-in launch can start in the tray.
pub const AUTOSTART_FLAG: &str = "--autostart";

/// `auto-launch` writes the Run value as `format!("{} {}", app_path, args)` with no
/// quoting — identical in 0.5.0 and in current 0.6.0 — and tauri-plugin-autostart hands
/// it a raw `current_exe()`. On a profile like `C:\Users\Raymundo Oyong\…`, Windows reads
/// the command as `C:\Users\Raymundo`: Task Manager labels the entry "Raymundo", and any
/// file of that name is launched instead of us. Re-quote the path, args stay outside.
///
/// Returns None when there is nothing to do, so this is safe to run on every launch.
fn repair_run_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.starts_with('"') {
        return None;
    }
    // Our arguments are all long flags, so the first " --" ends the path.
    let (path, args) = match value.find(" --") {
        Some(i) => (value[..i].trim_end(), &value[i..]),
        None => (value, ""),
    };
    if path.is_empty() {
        return None;
    }
    Some(format!("\"{path}\"{args}"))
}

/// Rewrite an existing unquoted Run entry in place. `true` if it changed.
///
/// Used both as a launch migration, so copies installed before this fix repair
/// themselves, and immediately after `enable()`, because the plugin writes the broken
/// form every single time.
pub fn repair_entry<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> std::io::Result<bool> {
    let name = &app.package_info().name;
    let run = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_SET_VALUE)?;
    let Ok(current) = run.get_value::<String, _>(name) else {
        return Ok(false); // not registered — nothing to repair
    };
    match repair_run_value(&current) {
        Some(fixed) => {
            run.set_value(name, &fixed)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[tauri::command]
pub fn get_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

/// Returns the state read back from the OS, never the state we asked for, so a
/// write that silently fails cannot present itself as success.
#[tauri::command]
pub fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<bool, String> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(|e| format!("enable failed: {e}"))?;
        // The plugin has just written an unquoted path; fix it before anything reads it.
        repair_entry(&app).map_err(|e| format!("enabled, but could not quote the path: {e}"))?;
    } else {
        mgr.disable().map_err(|e| format!("disable failed: {e}"))?;
    }
    let actual = mgr.is_enabled().map_err(|e| e.to_string())?;
    if actual != enabled {
        return Err(format!(
            "autostart still reads {actual} after asking for {enabled}"
        ));
    }
    Ok(actual)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reported case: an install path under a profile containing a space.
    const SPACED: &str = r"C:\Users\Raymundo Oyong\AppData\Local\PortPeek\portpeek.exe";

    #[test]
    fn quotes_an_install_path_containing_spaces() {
        let broken = format!("{SPACED} {AUTOSTART_FLAG}");
        assert_eq!(
            repair_run_value(&broken),
            Some(format!("\"{SPACED}\" {AUTOSTART_FLAG}")),
            "path must be quoted and the flag must stay outside the quotes"
        );
    }

    /// Without this, Windows resolves the entry to C:\Users\Raymundo.
    #[test]
    fn the_repaired_value_starts_with_the_whole_path_as_one_token() {
        let fixed = repair_run_value(&format!("{SPACED} {AUTOSTART_FLAG}")).unwrap();
        let first_token = fixed
            .strip_prefix('"')
            .and_then(|rest| rest.split('"').next())
            .unwrap();
        assert_eq!(first_token, SPACED);
    }

    #[test]
    fn leaves_an_already_quoted_entry_alone() {
        // The migration runs on every launch, so it must not churn the registry.
        let good = format!("\"{SPACED}\" {AUTOSTART_FLAG}");
        assert_eq!(repair_run_value(&good), None);
    }

    #[test]
    fn handles_a_spaced_path_with_no_arguments() {
        assert_eq!(repair_run_value(SPACED), Some(format!("\"{SPACED}\"")));
    }

    #[test]
    fn ignores_empty_and_whitespace_values() {
        assert_eq!(repair_run_value(""), None);
        assert_eq!(repair_run_value("   "), None);
    }
}
