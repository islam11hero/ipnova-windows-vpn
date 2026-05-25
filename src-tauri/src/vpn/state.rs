//! Shared VPN process state, paths, logging, and sing-box helpers.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use tauri::Manager;

#[cfg(windows)]
use crate::windows_proxy::{
    disable_system_proxy, harden_singbox_config, is_winhttp_proxy_for_port,
    is_wininet_proxy_for_port, prepare_system_proxy_config, reset_orphaned_proxies,
    SYSTEM_PROXY_PORT,
};
#[cfg(windows)]
use crate::windows_security::{kill_pid, spawn_singbox_elevated};

pub(crate) static VPN_CHILD: Mutex<Option<Child>> = Mutex::new(None);
#[cfg(windows)]
pub(crate) static VPN_EXTERNAL_PID: Mutex<Option<u32>> = Mutex::new(None);
#[cfg(windows)]
pub(crate) static VPN_SYSTEM_PROXY_ACTIVE: Mutex<bool> = Mutex::new(false);
#[cfg(windows)]
pub(crate) static VPN_PROXY_STATE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static VPN_ACTIVE_MODE: Mutex<Option<String>> = Mutex::new(None);

#[derive(Serialize, Clone)]
pub struct VpnRuntimeStatus {
    pub connected: bool,
    pub message: String,
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<crate::windows_proxy::ElevationNotice>,
}

#[derive(Serialize, Clone)]
pub struct VpnPreflight {
    pub ready: bool,
    pub singbox_found: bool,
    pub singbox_runnable: bool,
    pub port_available: bool,
    pub messages: Vec<String>,
}

pub fn vpn_state_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("vpn");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn resource_dir(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(dir) = app.path().resource_dir() {
        return dir.join("sing-box");
    }
    PathBuf::from("resources/sing-box")
}

pub fn singbox_binary(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(target_os = "windows")]
    let sub = "windows-amd64/sing-box.exe";
    #[cfg(target_os = "macos")]
    let sub = "macos-arm64/sing-box";
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let sub = "linux-amd64/sing-box";

    resource_dir(app).join(sub)
}

pub(crate) fn normalize_mode(mode: Option<String>) -> String {
    match mode.as_deref() {
        Some("system_proxy") | Some("proxy") => "system_proxy".into(),
        Some("tun") => "tun".into(),
        _ => "auto".into(),
    }
}

pub(crate) fn set_active_mode(mode: Option<&str>) {
    if let Ok(mut guard) = VPN_ACTIVE_MODE.lock() {
        *guard = mode.map(str::to_string);
    }
}

pub fn active_mode_label() -> Option<String> {
    VPN_ACTIVE_MODE.lock().ok().and_then(|g| g.clone())
}

fn ensure_tun_in_config(config: &mut Value) {
    let obj = match config.as_object_mut() {
        Some(o) => o,
        None => return,
    };

    let has_tun = obj.get("inbounds").and_then(|v| v.as_array()).is_some_and(|arr| {
        arr.iter().any(|entry| {
            entry
                .get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == "tun")
        })
    });

    if has_tun {
        return;
    }

    let tun = serde_json::json!({
        "type": "tun",
        "tag": "tun-in",
        "interface_name": "IPNOVA",
        "inet4_address": "172.19.0.1/30",
        "auto_route": true,
        "strict_route": false,
        "stack": "mixed",
        "sniff": true
    });

    match obj.get_mut("inbounds") {
        Some(Value::Array(arr)) => arr.push(tun),
        _ => {
            obj.insert("inbounds".into(), Value::Array(vec![tun]));
        }
    }
}

pub(crate) fn log_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn support_log_path(dir: &Path) -> PathBuf {
    dir.join("last-connect.log")
}

pub(crate) fn append_support_log(dir: &Path, event: &str, detail: &str) {
    let path = support_log_path(dir);
    let line = format!("[{}] {event}: {detail}\n", log_unix_secs());
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

pub(crate) fn tail_text_file(path: &Path, max_bytes: usize) -> String {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let slice = if data.len() > max_bytes {
        &data[data.len() - max_bytes..]
    } else {
        &data[..]
    };
    String::from_utf8_lossy(slice).into_owned()
}

pub fn singbox_process_running() -> bool {
    if let Ok(mut guard) = VPN_CHILD.lock() {
        if let Some(child) = guard.as_mut() {
            if child.try_wait().ok().flatten().is_none() {
                return true;
            }
            *guard = None;
        }
    }

    #[cfg(windows)]
    {
        if let Ok(pid_guard) = VPN_EXTERNAL_PID.lock() {
            if let Some(pid) = *pid_guard {
                let alive = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {pid}")])
                    .output()
                    .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
                    .unwrap_or(false);
                if alive {
                    return true;
                }
                if let Ok(mut g) = VPN_EXTERNAL_PID.lock() {
                    *g = None;
                }
            }
        }
    }

    false
}

#[cfg(windows)]
pub(crate) fn cleanup_active_proxy() {
    let flag_active = VPN_SYSTEM_PROXY_ACTIVE
        .lock()
        .ok()
        .is_some_and(|g| *g);
    let wininet_orphan = is_wininet_proxy_for_port(SYSTEM_PROXY_PORT);
    let winhttp_orphan = is_winhttp_proxy_for_port(SYSTEM_PROXY_PORT);
    let should = flag_active || wininet_orphan || winhttp_orphan;

    if !should {
        return;
    }
    let dir = VPN_PROXY_STATE_DIR.lock().ok().and_then(|g| g.clone());
    if let Some(d) = dir.as_ref() {
        let _ = disable_system_proxy(d);
        append_support_log(d, "cleanup", "restored system proxy after disconnect/crash");
    } else if wininet_orphan || winhttp_orphan {
        let (ie, wh) = reset_orphaned_proxies(SYSTEM_PROXY_PORT);
        let log_dir = std::env::temp_dir().join("ipnova-vpn-cleanup");
        let _ = std::fs::create_dir_all(&log_dir);
        append_support_log(
            &log_dir,
            "cleanup",
            &format!("orphan proxy reset without state dir (WinINet={ie}, WinHTTP={wh})"),
        );
    }
    if let Ok(mut g) = VPN_SYSTEM_PROXY_ACTIVE.lock() {
        *g = false;
    }
    if let Ok(mut g) = VPN_PROXY_STATE_DIR.lock() {
        *g = None;
    }
}

#[cfg(not(windows))]
pub(crate) fn cleanup_active_proxy() {}

/// Called on app exit — ensures WinINet/WinHTTP are not left pointing at 127.0.0.1:2080.
pub fn emergency_vpn_cleanup(app: &tauri::AppHandle) {
    if let Ok(dir) = vpn_state_dir(app) {
        append_support_log(&dir, "app_exit", "emergency cleanup");
    }
    let _ = stop_child();
    #[cfg(windows)]
    cleanup_active_proxy();
}

pub(crate) fn write_config(
    app: &tauri::AppHandle,
    mut config: Value,
    mode: &str,
) -> Result<PathBuf, String> {
    if mode == "system_proxy" {
        #[cfg(windows)]
        prepare_system_proxy_config(&mut config);
        #[cfg(not(windows))]
        return Err("System proxy mode is Windows-only".into());
    } else {
        ensure_tun_in_config(&mut config);
    }

    harden_singbox_config(&mut config);

    let dir = vpn_state_dir(app)?;
    let path = dir.join("config.json");
    let serialized = serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, serialized).map_err(|e| e.to_string())?;
    Ok(path)
}

pub(crate) fn stop_child() -> Result<(), String> {
    let mut guard = VPN_CHILD.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }

    #[cfg(windows)]
    {
        if let Ok(mut pid_guard) = VPN_EXTERNAL_PID.lock() {
            if let Some(pid) = pid_guard.take() {
                let _ = kill_pid(pid);
            }
        }
        cleanup_active_proxy();
    }

    set_active_mode(None);
    Ok(())
}

#[cfg(windows)]
pub(crate) fn wait_for_local_proxy(port: u16, timeout_ms: u64) -> bool {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    false
}

#[cfg(not(windows))]
fn wait_for_local_proxy(_port: u16, _timeout_ms: u64) -> bool {
    false
}

const PROBE_HOSTS: [(&str, &str); 3] = [
    ("api.ipify.org", "/"),
    ("ifconfig.me", "/ip"),
    ("icanhazip.com", "/"),
];

fn parse_ip_from_http_response(text: &str) -> Option<String> {
    let body = text.split("\r\n\r\n").nth(1)?;
    let ip = body.lines().next()?.trim();
    if ip.chars().all(|c| c.is_ascii_digit() || c == '.') && ip.contains('.') {
        Some(ip.to_string())
    } else {
        None
    }
}

fn http_get_via_local_proxy(port: u16, host: &str, path: &str) -> Option<String> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect_timeout(
        &addr.parse().ok()?,
        Duration::from_secs(15),
    )
    .ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));

    let req = format!(
        "GET http://{host}{path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    parse_ip_from_http_response(&String::from_utf8_lossy(&buf))
}

/// HTTP GET via local mixed proxy — verifies Marzban exit (multiple endpoints).
pub(crate) fn probe_exit_ip_via_local_proxy(port: u16) -> Option<String> {
    for (host, path) in PROBE_HOSTS {
        if let Some(ip) = http_get_via_local_proxy(port, host, path) {
            return Some(ip);
        }
    }
    None
}

pub(crate) fn spawn_singbox_process(
    binary: &Path,
    config_path: &Path,
    working_dir: &Path,
) -> Result<Child, String> {
    let stderr_log = working_dir.join("sing-box.stderr.log");
    let stderr_file = fs::File::create(&stderr_log).map_err(|e| e.to_string())?;

    let mut cmd = Command::new(binary);
    cmd.arg("run")
        .arg("-c")
        .arg(config_path)
        .arg("-D")
        .arg(working_dir)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    cmd.spawn().map_err(|e| e.to_string())
}
