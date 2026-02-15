use crate::capture::screen::{ScreenCaptureKitProvider, ScreenKeyframe, StubScreenCaptureKit};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderMode {
    Stub,
    Auto,
    Real,
}

impl ProviderMode {
    fn from_env() -> Self {
        match std::env::var("OPSCINEMA_PROVIDER_MODE")
            .unwrap_or_else(|_| "auto".to_string())
            .to_lowercase()
            .as_str()
        {
            "stub" => Self::Stub,
            "real" => Self::Real,
            _ => Self::Auto,
        }
    }
}

#[derive(Default)]
pub struct HybridScreenCaptureKitProvider;

impl ScreenCaptureKitProvider for HybridScreenCaptureKitProvider {
    fn capture_keyframe(&self, frame_ms: i64) -> anyhow::Result<ScreenKeyframe> {
        match ProviderMode::from_env() {
            ProviderMode::Stub => StubScreenCaptureKit.capture_keyframe(frame_ms),
            ProviderMode::Real => capture_real(frame_ms),
            ProviderMode::Auto => capture_real(frame_ms)
                .or_else(|_| capture_screencapture_cli(frame_ms))
                .or_else(|_| StubScreenCaptureKit.capture_keyframe(frame_ms)),
        }
    }
}

pub fn provider() -> Box<dyn ScreenCaptureKitProvider> {
    Box::new(HybridScreenCaptureKitProvider)
}

pub fn capture(frame_ms: i64) -> anyhow::Result<ScreenKeyframe> {
    provider().capture_keyframe(frame_ms)
}

fn capture_real(frame_ms: i64) -> anyhow::Result<ScreenKeyframe> {
    let helper = helper_script_path("screencapturekit_capture.swift");
    let png_path = std::env::temp_dir().join(format!("opscinema-sck-{}.png", uuid::Uuid::new_v4()));

    let output = Command::new("xcrun")
        .arg("swift")
        .arg(helper)
        .arg(&png_path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "ScreenCaptureKit helper failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let png_bytes = std::fs::read(&png_path)?;
    let _ = std::fs::remove_file(&png_path);

    let meta = capture_meta_from_env();
    Ok(ScreenKeyframe {
        frame_ms,
        display_id: meta.display_id,
        pixel_w: meta.pixel_w,
        pixel_h: meta.pixel_h,
        scale_factor: meta.scale_factor,
        png_bytes,
    })
}

fn capture_screencapture_cli(frame_ms: i64) -> anyhow::Result<ScreenKeyframe> {
    let png_path = std::env::temp_dir().join(format!("opscinema-sc-{}.png", uuid::Uuid::new_v4()));
    let status = Command::new("screencapture")
        .arg("-x")
        .arg(&png_path)
        .status()?;
    if !status.success() {
        anyhow::bail!("screencapture command failed")
    }
    let png_bytes = std::fs::read(&png_path)?;
    let _ = std::fs::remove_file(&png_path);

    let meta = capture_meta_from_env();
    Ok(ScreenKeyframe {
        frame_ms,
        display_id: meta.display_id,
        pixel_w: meta.pixel_w,
        pixel_h: meta.pixel_h,
        scale_factor: meta.scale_factor,
        png_bytes,
    })
}

fn helper_script_path(name: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.join("src/platform/macos/helpers").join(name)
}

struct CaptureMeta {
    display_id: String,
    pixel_w: u32,
    pixel_h: u32,
    scale_factor: String,
}

fn capture_meta_from_env() -> CaptureMeta {
    CaptureMeta {
        display_id: std::env::var("OPSCINEMA_CAPTURE_DISPLAY_ID")
            .unwrap_or_else(|_| "display.main".to_string()),
        pixel_w: std::env::var("OPSCINEMA_CAPTURE_PIXEL_W")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1920),
        pixel_h: std::env::var("OPSCINEMA_CAPTURE_PIXEL_H")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1080),
        scale_factor: std::env::var("OPSCINEMA_CAPTURE_SCALE")
            .unwrap_or_else(|_| "2.0".to_string()),
    }
}
