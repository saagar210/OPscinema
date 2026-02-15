use crate::capture::coord::normalize_bbox;
use opscinema_types::OcrBlock;

pub trait VisionProvider: Send + Sync {
    fn recognize(&self, png_bytes: &[u8]) -> anyhow::Result<Vec<OcrBlock>>;
}

#[derive(Default)]
pub struct StubVisionProvider;

impl VisionProvider for StubVisionProvider {
    fn recognize(&self, png_bytes: &[u8]) -> anyhow::Result<Vec<OcrBlock>> {
        let text = String::from_utf8_lossy(png_bytes);
        Ok(vec![OcrBlock {
            ocr_block_id: format!("ocr:{}", blake3::hash(png_bytes).to_hex()),
            bbox_norm: normalize_bbox(10.0, 10.0, 200.0, 60.0, 1920.0, 1080.0),
            text: text.to_string(),
            confidence: 90,
            language: Some("en".to_string()),
        }])
    }
}
