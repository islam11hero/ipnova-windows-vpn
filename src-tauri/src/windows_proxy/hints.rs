//! Environment hints (other VPNs, Edge policy, Firefox) — registry/tasklist, no PowerShell.

#[cfg(windows)]
use std::collections::HashSet;

#[cfg(windows)]
use super::process::hidden_command;

#[cfg(windows)]
const VPN_PROC_NEEDLES: &[&str] = &[
    "openvpn",
    "wireguard",
    "nordvpn",
    "expressvpn",
    "cisco",
    "anyconnect",
    "protonvpn",
    "surfshark",
    "windscribe",
    "mullvad",
    "hotspot",
    "zerotier",
    "tailscale",
];

#[cfg(windows)]
const VPN_ADAPTER_NEEDLES: &[&str] = &[
    "tap",
    "tun",
    "wireguard",
    "openvpn",
    "wintun",
    "vpn",
];

#[cfg(windows)]
pub fn detect_other_vpn_hints() -> Vec<String> {
    let mut hints: HashSet<String> = HashSet::new();
    hints.extend(detect_vpn_processes());
    hints.extend(detect_vpn_adapters());
    let mut out: Vec<String> = hints.into_iter().collect();
    out.sort();
    out
}

#[cfg(windows)]
fn detect_vpn_processes() -> Vec<String> {
    let output = hidden_command("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .output()
        .ok();
    let Some(output) = output else {
        return vec![];
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut found = Vec::new();
    for line in text.lines() {
        let name = line
            .trim()
            .trim_matches('"')
            .split(',')
            .next()
            .unwrap_or("")
            .trim_matches('"');
        if name.is_empty() {
            continue;
        }
        let lower = name.to_ascii_lowercase();
        if VPN_PROC_NEEDLES.iter().any(|n| lower.contains(n)) {
            found.push(format!("Process: {name}"));
        }
    }
    found
}

#[cfg(windows)]
fn detect_vpn_adapters() -> Vec<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let class = match hklm.open_subkey(
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}",
    ) {
        Ok(k) => k,
        Err(_) => return vec![],
    };

    let mut found = Vec::new();
    for index in class.enum_keys().flatten() {
        let Ok(adapter) = class.open_subkey(&index) else {
            continue;
        };
        let desc: String = adapter.get_value("DriverDesc").unwrap_or_default();
        if desc.is_empty() {
            continue;
        }
        let lower = desc.to_ascii_lowercase();
        if VPN_ADAPTER_NEEDLES.iter().any(|n| lower.contains(n)) {
            found.push(format!("Adapter: {desc}"));
        }
    }
    found
}

#[cfg(windows)]
pub fn detect_edge_proxy_policy() -> Option<String> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let mut lines = Vec::new();
    for (hive, label) in [
        (RegKey::predef(HKEY_LOCAL_MACHINE), "HKLM"),
        (RegKey::predef(HKEY_CURRENT_USER), "HKCU"),
    ] {
        let path = r"SOFTWARE\Policies\Microsoft\Edge";
        if let Ok(edge) = hive.open_subkey(path) {
            if let Ok(v) = edge.get_value::<String, _>("ProxySettings") {
                if !v.is_empty() {
                    lines.push(format!("{label}\\Policies\\Microsoft\\Edge : {v}"));
                }
            }
        }
    }
    let trimmed = lines.join("\n");
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("direct") && lower.contains("proxymode") {
        return Some(
            "Edge policy forces ProxyMode=direct — disable the Edge ProxySettings GPO or set ProxyMode=system."
                .into(),
        );
    }
    if lower.contains("fixed_servers") {
        return Some(
            "Edge policy uses fixed_servers — may override Windows system proxy. Set ProxyMode=system in GPO."
                .into(),
        );
    }
    Some(format!("Edge ProxySettings policy is set: {trimmed}"))
}

#[cfg(windows)]
pub fn detect_firefox_installed() -> bool {
    use std::path::Path;
    let paths = [
        std::env::var("ProgramFiles")
            .ok()
            .map(|p| Path::new(&p).join("Mozilla Firefox/firefox.exe")),
        std::env::var("ProgramFiles(x86)")
            .ok()
            .map(|p| Path::new(&p).join("Mozilla Firefox/firefox.exe")),
    ];
    paths.into_iter().flatten().any(|p| p.is_file())
}
