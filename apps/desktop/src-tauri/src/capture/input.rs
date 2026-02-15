use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClickEvent {
    pub frame_ms: i64,
    pub button: String,
    pub x_norm: u32,
    pub y_norm: u32,
    pub display_id: String,
}

pub fn capture_click(frame_ms: i64, display_id: &str) -> ClickEvent {
    let x_norm = std::env::var("OPSCINEMA_CLICK_X_NORM")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5_000)
        .min(10_000);
    let y_norm = std::env::var("OPSCINEMA_CLICK_Y_NORM")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5_000)
        .min(10_000);

    let button = std::env::var("OPSCINEMA_CLICK_BUTTON")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "left".to_string());

    ClickEvent {
        frame_ms,
        button,
        x_norm,
        y_norm,
        display_id: display_id.to_string(),
    }
}
