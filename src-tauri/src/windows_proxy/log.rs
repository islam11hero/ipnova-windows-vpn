//! Append-only support log for proxy operations.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn append_proxy_log(state_dir: &Path, event: &str, detail: &str) {
    let _ = std::fs::create_dir_all(state_dir);
    let path = state_dir.join("last-connect.log");
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] proxy/{event}: {detail}\n");
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}
