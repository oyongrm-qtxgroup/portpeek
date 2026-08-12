use std::os::windows::process::CommandExt;
use std::process::Command;

use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Without this the console flashes a window on every kill (PRD §4/§8).
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Parents that mean "the listener is a child of a launcher", so terminating the
/// listening PID alone is likely to leave the real server running (PRD §8).
const LAUNCHERS: &[&str] = &[
    "cmd.exe",
    "powershell.exe",
    "pwsh.exe",
    "npm.cmd",
    "npm.exe",
    "pnpm.cmd",
    "pnpm.exe",
    "yarn.cmd",
    "yarn.exe",
    "bun.exe",
    "bunx.exe",
    "npx.cmd",
    "bash.exe",
    "sh.exe",
];

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
    pub parent_name: Option<String>,
    pub child_count: usize,
    /// Drives the default state of the "also terminate child processes" checkbox.
    pub suggest_tree: bool,
}

fn suggests_tree(parent: Option<&str>, children: usize) -> bool {
    children > 0
        || parent
            .map(|n| LAUNCHERS.contains(&n.to_ascii_lowercase().as_str()))
            .unwrap_or(false)
}

/// Deliberately does a FULL process refresh, unlike the 2s poll which is scoped to
/// listening PIDs only. Children of the target are not listeners themselves, so
/// they are invisible to the poll's narrow refresh. This runs once, on opening the
/// confirm dialog, so the hot path stays cheap.
#[tauri::command]
pub fn inspect_process(pid: u32) -> Result<ChainInfo, String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing(),
    );

    let target = Pid::from_u32(pid);
    let parent_name = sys
        .process(target)
        .and_then(|p| p.parent())
        .and_then(|parent| sys.process(parent))
        .map(|p| p.name().to_string_lossy().into_owned());
    let child_count = sys
        .processes()
        .values()
        .filter(|p| p.parent() == Some(target))
        .count();

    Ok(ChainInfo {
        suggest_tree: suggests_tree(parent_name.as_deref(), child_count),
        parent_name,
        child_count,
    })
}

/// (denied, already_gone) from taskkill's own text. Exit codes are not used —
/// see kill_process. Strings are matched case-insensitively against the real
/// wording taskkill emits, captured from this machine and pinned in the tests.
fn classify_output(text: &str) -> (bool, bool) {
    let lower = text.to_ascii_lowercase();
    (
        lower.contains("access is denied"),
        lower.contains("not found") || lower.contains("no running instance"),
    )
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KillReport {
    pub denied: bool,
    pub already_gone: bool,
    pub detail: String,
}

/// Issues the termination and reports only what it can say for certain.
///
/// The caller must NOT treat this returning Ok as "the port is free" — taskkill's
/// exit code lies often enough that PRD §8 mandates verifying by port state on the
/// next poll instead. This return value exists to explain FAILURES (access denied,
/// already exited), never to claim success.
#[tauri::command]
pub fn kill_process(pid: u32, tree: bool) -> Result<KillReport, String> {
    // Server-side shield. The UI hides these rows' kill buttons, but a renderer bug
    // or a crafted invoke must never be able to reach taskkill for PID 0/4.
    if pid == 0 || pid == 4 {
        return Err(
            "PID 0 and PID 4 are protected Windows system processes and are never terminated."
                .into(),
        );
    }

    let mut cmd = Command::new("taskkill");
    cmd.arg("/PID").arg(pid.to_string()).arg("/F");
    if tree {
        // /T takes the whole tree. sysinfo::kill() is taskkill /F WITHOUT /T, which
        // is exactly the case that leaves a dev server's children holding the port.
        cmd.arg("/T");
    }
    cmd.creation_flags(CREATE_NO_WINDOW);

    let out = cmd
        .output()
        .map_err(|e| format!("could not run taskkill: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let (denied, already_gone) = classify_output(&text);

    Ok(KillReport {
        denied,
        already_gone,
        detail: text.trim().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_is_suggested_for_launcher_chains() {
        // The real specimen on this machine: bunx.exe -> node.exe holding :8080.
        assert!(suggests_tree(Some("bunx.exe"), 0));
        assert!(suggests_tree(Some("cmd.exe"), 0));
        assert!(suggests_tree(Some("NPM.CMD"), 0), "casing must not matter");
        // Having children is enough on its own, whatever the parent is.
        assert!(suggests_tree(Some("explorer.exe"), 2));
        assert!(suggests_tree(None, 1));
        // A standalone server with no children and a plain parent: single kill.
        assert!(!suggests_tree(Some("explorer.exe"), 0));
        assert!(!suggests_tree(None, 0));
    }

    /// Verbatim taskkill output captured on this machine — not paraphrased, so the
    /// §8 failure toasts key off what taskkill actually prints.
    #[test]
    fn failure_paths_are_read_from_taskkills_real_wording() {
        let denied = "ERROR: The process with PID 4 could not be terminated.\r\nReason: Access is denied.";
        assert_eq!(classify_output(denied), (true, false));

        let missing = "ERROR: The process \"999999\" not found.";
        assert_eq!(classify_output(missing), (false, true));

        // A successful kill must trip NEITHER flag, or we would report a false
        // failure on the happy path.
        let ok = "SUCCESS: The process with PID 43100 has been terminated.";
        assert_eq!(classify_output(ok), (false, false));
    }

    #[test]
    fn pid_0_and_4_are_refused_before_reaching_taskkill() {
        assert!(kill_process(0, false).is_err());
        assert!(kill_process(4, true).is_err());
    }
}
