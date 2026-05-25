//! Built-in troubleshooting checklist (Settings / support).

#[cfg(windows)]
use std::path::Path;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
use super::scenarios::{evaluate_proxy_scenarios, gather_scenario_inputs, scenarios_to_troubleshooting_checks};

#[derive(serde::Serialize, Clone)]
pub struct TroubleshootingCheck {
    pub id: String,
    pub title: String,
    pub status: String,
    pub detail: String,
}

#[cfg(windows)]
fn singbox_runnable(singbox_path: &Path, singbox_exists: bool) -> bool {
    if !singbox_exists {
        return false;
    }
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new(singbox_path)
        .arg("version")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn troubleshooting_checks(singbox_path: &Path, singbox_exists: bool) -> Vec<TroubleshootingCheck> {
    let runnable = singbox_runnable(singbox_path, singbox_exists);
    let input = gather_scenario_inputs(false, singbox_exists, runnable, None);
    let report = evaluate_proxy_scenarios(&input);
    let mut checks = scenarios_to_troubleshooting_checks(&report);

    for check in &mut checks {
        if check.id == "singbox_file" && singbox_exists {
            check.detail = singbox_path.display().to_string();
        }
    }
    checks
}
