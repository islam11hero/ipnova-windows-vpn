//! TCP / netstat probes without PowerShell modules.

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[cfg(windows)]
use super::process::hidden_command;

/// True if something accepts TCP on loopback:port.
pub fn is_local_proxy_listening(port: u16) -> bool {
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("valid loopback");
    TcpStream::connect_timeout(&addr, Duration::from_millis(400)).is_ok()
}

/// Owning process for a listening port via `netstat -ano` (no PowerShell).
#[cfg(windows)]
pub fn port_listener_info(port: u16) -> (Option<u32>, String) {
    let loopback_needle = format!("127.0.0.1:{port}");
    let wildcard_needle = format!("0.0.0.0:{port}");
    let output = hidden_command("netstat")
        .args(["-ano", "-p", "TCP"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();

    for line in output.lines() {
        let upper = line.to_ascii_uppercase();
        let has_port = line.contains(&loopback_needle)
            || line.contains(&wildcard_needle)
            || netstat_line_has_listen_port(line, port);
        if !upper.contains("LISTENING") || !has_port {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if let Some(pid_str) = parts.last() {
            if let Ok(pid) = pid_str.parse::<u32>() {
                if pid > 0 {
                    let name = process_name(pid).unwrap_or_default();
                    return (Some(pid), name);
                }
            }
        }
    }
    (None, String::new())
}

/// Match LISTEN row for exact port (avoids `:2080` matching `:20801`).
#[cfg(windows)]
fn netstat_line_has_listen_port(line: &str, port: u16) -> bool {
    let token = format!(":{port}");
    for part in line.split_whitespace() {
        if part.ends_with(&token) {
            return true;
        }
    }
    false
}

#[cfg(windows)]
fn process_name(pid: u32) -> Option<String> {
    let out = hidden_command("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output()
        .ok()?;
    let line = String::from_utf8_lossy(&out.stdout);
    let first = line.lines().next()?;
    let name = first.trim_matches('"').split(',').next()?;
    Some(name.trim_matches('"').to_string())
}

#[cfg(not(windows))]
pub fn port_listener_info(_port: u16) -> (Option<u32>, String) {
    (None, String::new())
}

#[cfg(windows)]
pub fn preflight_port_message(port: u16, singbox_running: bool) -> Option<String> {
    if singbox_running || !is_local_proxy_listening(port) {
        return None;
    }
    let (pid, name) = port_listener_info(port);
    let is_singbox = name.to_ascii_lowercase().contains("sing-box");
    if is_singbox {
        return None;
    }
    Some(match (pid, name.is_empty()) {
        (Some(p), false) => format!("Port {port} is used by {name} (PID {p}) — close it or restart IPNOVA"),
        (Some(p), true) => format!("Port {port} is used by PID {p} — close it or restart IPNOVA"),
        _ => format!("Port {port} is in use — close other VPN apps or restart IPNOVA"),
    })
}

#[cfg(not(windows))]
pub fn preflight_port_message(_port: u16, _singbox_running: bool) -> Option<String> {
    None
}
