//! Enable / disable system proxy (WinINet + WinHTTP orchestration).

#[cfg(windows)]
use std::path::Path;

#[cfg(windows)]
use super::backup::{backup_proxy_settings, restore_or_disable};
#[cfg(windows)]
use super::constants::SYSTEM_PROXY_PORT;
#[cfg(windows)]
use super::log::append_proxy_log;
#[cfg(windows)]
use super::notify::notify_proxy_change;
#[cfg(windows)]
use super::winhttp::{enable_winhttp_proxy, is_winhttp_proxy_for_port, reset_winhttp_stack};
#[cfg(windows)]
use super::wininet::{enable_wininet_proxy_verified, reset_wininet_if_orphaned};

#[cfg(windows)]
use super::types::ProxyEnableResult;

#[cfg(windows)]
pub fn enable_system_proxy(state_dir: &Path) -> Result<ProxyEnableResult, String> {
    match backup_proxy_settings(state_dir) {
        Ok(()) => append_proxy_log(state_dir, "backup", "ok"),
        Err(e) => append_proxy_log(
            state_dir,
            "backup_fail",
            &format!("{e} — disconnect may not restore previous proxy"),
        ),
    }

    enable_wininet_proxy_verified(SYSTEM_PROXY_PORT, state_dir)?;

    let winhttp_result = enable_winhttp_proxy(state_dir, SYSTEM_PROXY_PORT);
    let (winhttp_ok, winhttp_method, winhttp_warning) = match winhttp_result {
        Ok(method) => {
            append_proxy_log(state_dir, "winhttp_ok", &method);
            (true, Some(method), None)
        }
        Err(e) => {
            append_proxy_log(state_dir, "winhttp_fail", &e);
            (false, None, Some(e))
        }
    };
    notify_proxy_change();
    Ok(ProxyEnableResult {
        winhttp_ok,
        winhttp_warning,
        winhttp_method,
    })
}

#[cfg(windows)]
pub fn disable_system_proxy(state_dir: &Path) -> Result<(), String> {
    restore_or_disable(state_dir);
    notify_proxy_change();
    Ok(())
}

#[cfg(windows)]
pub fn reset_winhttp_if_orphaned(port: u16) -> bool {
    if !is_winhttp_proxy_for_port(port) {
        return false;
    }
    reset_winhttp_stack();
    !is_winhttp_proxy_for_port(port)
}

/// Clear WinINet + WinHTTP orphans after crash (no sing-box, proxy still on our port).
#[cfg(windows)]
pub fn reset_orphaned_proxies(port: u16) -> (bool, bool) {
    let ie = reset_wininet_if_orphaned(port);
    let wh = reset_winhttp_if_orphaned(port);
    if ie || wh {
        notify_proxy_change();
    }
    (ie, wh)
}
