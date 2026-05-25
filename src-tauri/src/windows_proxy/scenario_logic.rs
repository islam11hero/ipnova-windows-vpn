//! Pure proxy scenario logic (unit-tested on every OS).

use super::types::ProxyConflictCheck;

/// WinINet snapshot used across conflict + scenario evaluation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WininetSnapshot {
    pub enabled: bool,
    pub server: String,
    pub auto_detect: bool,
    pub pac_url: String,
}

impl WininetSnapshot {
    pub fn from_parts(enabled: bool, server: impl Into<String>, auto_detect: bool, pac_url: impl Into<String>) -> Self {
        Self {
            enabled,
            server: server.into(),
            auto_detect,
            pac_url: pac_url.into(),
        }
    }
}

/// True when `ProxyServer` points at loopback on `port` (all common WinINet formats).
pub fn server_points_at_local_port(server: &str, port: u16) -> bool {
    let server = server.trim();
    if server.is_empty() {
        return false;
    }
    let lower = server.to_ascii_lowercase();
    if !lower.contains("127.0.0.1") {
        return false;
    }
    let token = format!("127.0.0.1:{port}");
    if contains_port_token(&lower, &token) {
        return true;
    }
    // http=127.0.0.1:2080;https=127.0.0.1:2080;...
    let colon_port = format!(":{port}");
    contains_port_token(&lower, &colon_port)
}

/// Match `:2080` but not `:20801`.
fn contains_port_token(haystack: &str, token: &str) -> bool {
    let mut start = 0;
    while let Some(rel) = haystack[start..].find(token) {
        let idx = start + rel;
        let after = idx + token.len();
        if after >= haystack.len() || !haystack.as_bytes()[after].is_ascii_digit() {
            return true;
        }
        start = after;
    }
    false
}

/// Manual proxy set to something other than our local mixed inbound.
pub fn is_foreign_manual_proxy(snapshot: &WininetSnapshot, port: u16) -> bool {
    snapshot.enabled
        && !snapshot.server.trim().is_empty()
        && !server_points_at_local_port(&snapshot.server, port)
}

/// PAC URL or WPAD auto-detect enabled.
pub fn has_pac_or_wpad(snapshot: &WininetSnapshot) -> bool {
    snapshot.auto_detect || !snapshot.pac_url.trim().is_empty()
}

/// Build pre-connect conflict summary (shared with `check_proxy_conflict`).
pub fn evaluate_proxy_conflict(snapshot: &WininetSnapshot, port: u16) -> ProxyConflictCheck {
    let server = snapshot.server.trim().to_string();
    let pac_url = snapshot.pac_url.trim().to_string();
    let auto_detect = snapshot.auto_detect;
    let foreign_proxy = is_foreign_manual_proxy(snapshot, port);
    let has_pac = has_pac_or_wpad(snapshot);
    let has_conflict = foreign_proxy || has_pac;
    let port_s = port.to_string();

    let mut message = String::new();
    if foreign_proxy {
        message = format!(
            "Windows proxy is set to «{server}». IPNOVA will replace it with 127.0.0.1:{port_s} while connected (restored on disconnect)."
        );
    }
    if has_pac {
        let pac_note = if pac_url.is_empty() {
            "PAC/WPAD auto-detect is enabled".to_string()
        } else {
            format!("PAC URL is set ({pac_url})")
        };
        if message.is_empty() {
            message = format!(
                "{pac_note} — IPNOVA will disable it and use 127.0.0.1:{port_s} while connected."
            );
        } else {
            message.push_str(&format!(" Also: {pac_note}."));
        }
    }
    if snapshot.enabled && server.is_empty() && !has_pac {
        message = "Proxy is enabled but empty — IPNOVA will set local proxy on connect.".into();
    }

    ProxyConflictCheck {
        has_conflict,
        proxy_enabled: snapshot.enabled,
        current_proxy_server: server,
        has_pac,
        pac_url,
        auto_detect,
        message,
    }
}

/// Parse `netsh winhttp show proxy` text (shared with backup restore).
pub fn parse_winhttp_show_proxy(output: &str) -> (bool, Option<String>, Option<String>) {
    let direct = output.contains("Direct access");
    let mut proxy = None;
    let mut bypass = None;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Proxy Server") {
            if let Some((_, v)) = trimmed.split_once(':') {
                let v = v.trim();
                if !v.is_empty() {
                    proxy = Some(v.to_string());
                }
            }
        } else if trimmed.starts_with("Bypass List") {
            if let Some((_, v)) = trimmed.split_once(':') {
                let v = v.trim();
                if !v.is_empty() {
                    bypass = Some(v.to_string());
                }
            }
        }
    }
    (direct, proxy, bypass)
}

/// WinHTTP / advproxy output contains our loopback port.
pub fn netsh_output_points_at_port(output: &str, port: u16) -> bool {
    let port_s = port.to_string();
    output.contains("127.0.0.1") && output.contains(&port_s)
}

/// Process name looks like sing-box (mixed inbound owner).
pub fn is_singbox_process_name(name: &str) -> bool {
    let n = name.trim().to_ascii_lowercase();
    n.contains("sing-box") || n == "singbox"
}

/// Port 2080 held by another app → blocks Connect.
pub fn port_holder_blocks_connect(process_name: &str, singbox_running: bool) -> bool {
    if singbox_running {
        return false;
    }
    if process_name.is_empty() {
        return true;
    }
    !is_singbox_process_name(process_name)
}

/// WcmSvc depends on WinHttpAutoProxySvc while the latter is Disabled (24H2 Wi‑Fi issue).
pub fn wcm_24h2_issue(wcm_deps: &str, autoproxy_start: &str) -> bool {
    wcm_deps.contains("WinHttpAutoProxySvc")
        && autoproxy_start.eq_ignore_ascii_case("disabled")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_points_simple_and_combined() {
        assert!(server_points_at_local_port("127.0.0.1:2080", 2080));
        assert!(server_points_at_local_port(
            "http=127.0.0.1:2080;https=127.0.0.1:2080;socks=127.0.0.1:2080",
            2080
        ));
        assert!(!server_points_at_local_port("10.0.0.1:8080", 2080));
        assert!(!server_points_at_local_port("", 2080));
        assert!(!server_points_at_local_port("127.0.0.1:1080", 2080));
        assert!(!server_points_at_local_port("127.0.0.1:20801", 2080));
    }

    #[test]
    fn foreign_proxy_and_pac() {
        let snap = WininetSnapshot::from_parts(true, "proxy.corp:8080", false, "");
        assert!(is_foreign_manual_proxy(&snap, 2080));
        let pac = WininetSnapshot::from_parts(false, "", true, "");
        assert!(has_pac_or_wpad(&pac));
        let pac_url = WininetSnapshot::from_parts(false, "", false, "http://wpad/wpad.dat");
        assert!(has_pac_or_wpad(&pac_url));
    }

    #[test]
    fn conflict_messages() {
        let snap = WininetSnapshot::from_parts(true, "1.2.3.4:3128", false, "");
        let c = evaluate_proxy_conflict(&snap, 2080);
        assert!(c.has_conflict);
        assert!(c.message.contains("1.2.3.4:3128"));
    }

    #[test]
    fn parse_winhttp_direct_and_proxy() {
        let sample = r#"
Current WinHTTP proxy settings:

    Direct access (no proxy server).
"#;
        let (direct, proxy, bypass) = parse_winhttp_show_proxy(sample);
        assert!(direct);
        assert!(proxy.is_none());
        assert!(bypass.is_none());

        let sample2 = r#"
Proxy Server(s)  :  127.0.0.1:2080
    Bypass List     :  <local>
"#;
        let (direct2, proxy2, _) = parse_winhttp_show_proxy(sample2);
        assert!(!direct2);
        assert_eq!(proxy2.as_deref(), Some("127.0.0.1:2080"));
    }

    #[test]
    fn port_holder_and_singbox_name() {
        assert!(is_singbox_process_name("sing-box"));
        assert!(is_singbox_process_name("sing-box.exe"));
        assert!(!is_singbox_process_name("chrome"));
        assert!(!port_holder_blocks_connect("sing-box", false));
        assert!(port_holder_blocks_connect("openvpn", false));
        assert!(!port_holder_blocks_connect("openvpn", true));
    }

    #[test]
    fn wcm_dependency_detect() {
        assert!(wcm_24h2_issue("RpcSs\0WinHttpAutoProxySvc\0", "Disabled"));
        assert!(!wcm_24h2_issue("RpcSs", "Manual"));
    }
}
