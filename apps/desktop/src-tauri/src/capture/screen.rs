use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenKeyframe {
    pub frame_ms: i64,
    pub display_id: String,
    pub pixel_w: u32,
    pub pixel_h: u32,
    pub scale_factor: String,
    pub png_bytes: Vec<u8>,
}

pub trait ScreenCaptureKitProvider: Send + Sync {
    fn capture_keyframe(&self, frame_ms: i64) -> anyhow::Result<ScreenKeyframe>;
}

#[derive(Default)]
pub struct StubScreenCaptureKit;

impl ScreenCaptureKitProvider for StubScreenCaptureKit {
    fn capture_keyframe(&self, frame_ms: i64) -> anyhow::Result<ScreenKeyframe> {
        Ok(ScreenKeyframe {
            frame_ms,
            display_id: "display.main".to_string(),
            pixel_w: 1920,
            pixel_h: 1080,
            scale_factor: "2.0".to_string(),
            png_bytes: format!("fake-keyframe-{frame_ms}").into_bytes(),
        })
    }
}
