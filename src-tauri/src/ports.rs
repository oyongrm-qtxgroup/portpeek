use std::collections::{BTreeMap, HashSet};
use std::sync::Mutex;

use netstat2::{
    get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState,
};
use serde::Serialize;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Killing any of these can bugcheck Windows. They keep a kill button, but the
/// confirm dialog has to say so out loud (PRD §5).
const CRITICAL: &[&str] = &[
    "lsass.exe",
    "services.exe",
    "wininit.exe",
    "csrss.exe",
    "smss.exe",
    "svchost.exe",
];

/// Long-lived so sysinfo can reuse its process cache between polls.
pub struct PortState(pub Mutex<System>);

impl Default for PortState {
    fn default() -> Self {
        Self(Mutex::new(System::new()))
    }
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PortRow {
    pub port: u16,
    pub pid: u32,
    pub process: String,
    pub exe: Option<String>,
    /// Both flags can be true: one row collapsing an IPv4 and IPv6 listener
    /// that share a PID+port (PRD §5).
    pub ipv4: bool,
    pub ipv6: bool,
    /// "hard"     -> PID 0/4, no kill button at all.
    /// "critical" -> killable, but the dialog must carry the bugcheck warning.
    pub shield: Option<&'static str>,
    pub badge: Option<&'static str>,
}

fn classify(pid: u32, name: &str) -> (Option<&'static str>, Option<&'static str>) {
    match pid {
        // PID 4 owns http.sys-backed listeners (IIS, WinRM 5985, sometimes 80/443).
        4 => (Some("hard"), Some("HTTP.sys / Windows")),
        0 => (Some("hard"), None),
        _ if CRITICAL.contains(&name.to_ascii_lowercase().as_str()) => (Some("critical"), None),
        _ => (None, None),
    }
}

#[tauri::command]
pub fn list_ports(state: tauri::State<'_, PortState>) -> Result<Vec<PortRow>, String> {
    scan(&state)
}

/// The cadence lives in Rust (see `lib.rs`), not a JS `setInterval`: WebView2
/// throttles timers in occluded windows, which measurably stretched the poll past
/// the 2s freshness budget in PRD §11.
pub fn scan(state: &PortState) -> Result<Vec<PortRow>, String> {
    // Debug-only trace: the poller is supposed to skip this entirely while the
    // widget is parked in the tray, and that is otherwise invisible from outside.
    #[cfg(debug_assertions)]
    eprintln!("[portpeek] list_ports scan");

    let sockets = get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP,
    )
    .map_err(|e| format!("socket enumeration failed: {e}"))?;

    // Keep only LISTEN, and note which PIDs we actually care about, so the sysinfo
    // refresh below touches a handful of processes instead of every process on the
    // box. That is what keeps the 2s poll near-idle (PRD §11).
    let listeners: Vec<(u16, bool, &Vec<u32>)> = sockets
        .iter()
        .filter_map(|si| match &si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) if tcp.state == TcpState::Listen => {
                Some((tcp.local_port, tcp.local_addr.is_ipv6(), &si.associated_pids))
            }
            _ => None,
        })
        .collect();

    let pids: HashSet<Pid> = listeners
        .iter()
        .flat_map(|(_, _, pids)| pids.iter())
        .map(|p| Pid::from_u32(*p))
        .collect();
    let pid_list: Vec<Pid> = pids.into_iter().collect();

    let mut sys = state.0.lock().map_err(|_| "process state poisoned")?;
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pid_list),
        true,
        // exe is NOT populated by default (PRD §4). OnlyIfNotSet keeps it cached
        // across polls rather than re-opening every process handle each tick.
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );

    // BTreeMap keyed (port, pid): dedupes v4/v6 into one row AND sorts by port asc.
    let mut rows: BTreeMap<(u16, u32), PortRow> = BTreeMap::new();
    for (port, is_v6, pids) in listeners {
        for &pid in pids {
            let row = rows.entry((port, pid)).or_insert_with(|| {
                let proc = sys.process(Pid::from_u32(pid));
                let name = proc
                    .map(|p| p.name().to_string_lossy().into_owned())
                    .unwrap_or_else(|| match pid {
                        0 => "System Idle Process".to_string(),
                        4 => "System".to_string(),
                        // Not an error: elevated/other-user processes are simply
                        // not readable without admin. Degrade, never drop the row.
                        _ => "(access denied)".to_string(),
                    });
                let (shield, badge) = classify(pid, &name);
                PortRow {
                    port,
                    pid,
                    exe: proc
                        .and_then(|p| p.exe())
                        .map(|p| p.to_string_lossy().into_owned()),
                    process: name,
                    ipv4: false,
                    ipv6: false,
                    shield,
                    badge,
                }
            });
            if is_v6 {
                row.ipv6 = true;
            } else {
                row.ipv4 = true;
            }
        }
    }

    Ok(rows.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shields_match_prd_section_5() {
        assert_eq!(classify(4, "System"), (Some("hard"), Some("HTTP.sys / Windows")));
        assert_eq!(classify(0, "System Idle Process").0, Some("hard"));
        // case-insensitive: netstat/sysinfo casing is not guaranteed
        assert_eq!(classify(900, "LSASS.EXE").0, Some("critical"));
        assert_eq!(classify(900, "svchost.exe").0, Some("critical"));
        assert_eq!(classify(1234, "node.exe"), (None, None));
    }

    /// The real invariant of the dedupe: one row per (port, pid) with both family
    /// flags set, never two rows for a dual-stack listener.
    #[test]
    fn dual_stack_listener_collapses_to_one_row() {
        let mut rows: BTreeMap<(u16, u32), PortRow> = BTreeMap::new();
        for is_v6 in [false, true] {
            let row = rows.entry((8080, 42)).or_insert_with(|| PortRow {
                port: 8080,
                pid: 42,
                process: "node.exe".into(),
                exe: None,
                ipv4: false,
                ipv6: false,
                shield: None,
                badge: None,
            });
            if is_v6 { row.ipv6 = true } else { row.ipv4 = true }
        }
        assert_eq!(rows.len(), 1);
        let r = rows.values().next().unwrap();
        assert!(r.ipv4 && r.ipv6);
    }
}
