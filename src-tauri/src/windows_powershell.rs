//! Shared PowerShell runner for Windows proxy/security modules.

#[cfg(windows)]
const PS_TIMEOUT_SECS: u64 = 45;

#[cfg(windows)]
fn run_powershell_impl(executable: &str, script: &str) -> Result<String, String> {
    use std::process::Command;

    let child = Command::new(executable)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .map_err(|e| format!("Failed to start {executable}: {e}"))?;

    let stdout = String::from_utf8_lossy(&child.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&child.stderr).trim().to_string();

    if child.status.success() {
        Ok(stdout)
    } else if !stderr.is_empty() {
        Err(stderr)
    } else {
        Err(format!("{executable} exit {}", child.status))
    }
}

#[cfg(windows)]
pub fn run_powershell_with_timeout(script: &str, timeout_secs: u64) -> Result<String, String> {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let script_owned = script.to_string();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let primary = run_powershell_impl("powershell", &script_owned);
        let result = match primary {
            Ok(out) => Ok(out),
            Err(ps_err) => match run_powershell_impl("pwsh", &script_owned) {
                Ok(out) => Ok(out),
                Err(pwsh_err) => Err(format!("powershell: {ps_err}; pwsh: {pwsh_err}")),
            },
        };
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(timeout_secs.max(5))) {
        Ok(r) => r,
        Err(_) => Err(format!("PowerShell timed out after {timeout_secs}s")),
    }
}

#[cfg(windows)]
pub fn run_powershell(script: &str) -> Result<String, String> {
    run_powershell_with_timeout(script, PS_TIMEOUT_SECS)
}

#[cfg(not(windows))]
pub fn run_powershell(_script: &str) -> Result<String, String> {
    Err("PowerShell is only available on Windows".into())
}
