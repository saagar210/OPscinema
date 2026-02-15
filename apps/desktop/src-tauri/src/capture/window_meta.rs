use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowMeta {
    pub frame_ms: i64,
    pub frontmost_bundle_id: Option<String>,
    pub frontmost_title: Option<String>,
}

pub fn capture_window_meta(frame_ms: i64) -> WindowMeta {
    let bundle = std::env::var("OPSCINEMA_FRONTMOST_BUNDLE")
        .ok()
        .or_else(frontmost_bundle_id_from_macos);
    let title = std::env::var("OPSCINEMA_FRONTMOST_TITLE")
        .ok()
        .or_else(frontmost_title_from_macos);

    WindowMeta {
        frame_ms,
        frontmost_bundle_id: bundle,
        frontmost_title: title,
    }
}

fn frontmost_bundle_id_from_macos() -> Option<String> {
    run_osascript("tell application \"System Events\" to get bundle identifier of first application process whose frontmost is true")
}

fn frontmost_title_from_macos() -> Option<String> {
    run_osascript("tell application \"System Events\" to get name of first window of (first application process whose frontmost is true)")
}

fn run_osascript(script: &str) -> Option<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
