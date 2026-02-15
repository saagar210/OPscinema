use crate::ocr::vision::{StubVisionProvider, VisionProvider};
use opscinema_types::{BBoxNorm, OcrBlock};
use serde::Deserialize;
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
struct HybridVisionProvider;

impl VisionProvider for HybridVisionProvider {
    fn recognize(&self, png_bytes: &[u8]) -> anyhow::Result<Vec<OcrBlock>> {
        if let Ok(raw) = std::env::var("OPSCINEMA_VISION_RAW_JSON") {
            return parse_provider_blocks(raw.as_bytes());
        }

        match ProviderMode::from_env() {
            ProviderMode::Stub => StubVisionProvider.recognize(png_bytes),
            ProviderMode::Real => recognize_real(png_bytes),
            ProviderMode::Auto => {
                recognize_real(png_bytes).or_else(|_| StubVisionProvider.recognize(png_bytes))
            }
        }
    }
}

pub fn provider() -> Box<dyn VisionProvider> {
    Box::new(HybridVisionProvider)
}

#[derive(Debug, Deserialize)]
struct VisionBlock {
    text: String,
    confidence: f64,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn recognize_real(png_bytes: &[u8]) -> anyhow::Result<Vec<OcrBlock>> {
    let helper = helper_script_path("vision_ocr.swift");
    let mut child = Command::new("xcrun")
        .arg("swift")
        .arg(helper)
        .arg("--stdin-image")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(png_bytes)?;
    }
    let output = child.wait_with_output()?;

    if !output.status.success() {
        anyhow::bail!(
            "Vision helper failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    parse_provider_blocks(&output.stdout)
}

pub(crate) fn parse_provider_blocks(raw: &[u8]) -> anyhow::Result<Vec<OcrBlock>> {
    let blocks: Vec<VisionBlock> = serde_json::from_slice(raw)?;
    let mut out = Vec::with_capacity(blocks.len());
    for b in blocks {
        if b.text.trim().is_empty() {
            anyhow::bail!("provider schema invalid: text cannot be empty")
        }
        if !(0.0..=1.0).contains(&b.confidence) {
            anyhow::bail!("provider schema invalid: confidence out of range")
        }
        if !is_norm(b.x) || !is_norm(b.y) || !is_norm(b.w) || !is_norm(b.h) {
            anyhow::bail!("provider schema invalid: bbox out of range")
        }
        if b.w <= 0.0 || b.h <= 0.0 {
            anyhow::bail!("provider schema invalid: bbox must be positive")
        }
        if b.x + b.w > 1.0 + f64::EPSILON || b.y + b.h > 1.0 + f64::EPSILON {
            anyhow::bail!("provider schema invalid: bbox exceeds normalized bounds")
        }

        out.push(OcrBlock {
            ocr_block_id: format!("ocr:{}", blake3::hash(b.text.as_bytes()).to_hex()),
            text: b.text,
            confidence: (b.confidence * 100.0).round() as u8,
            language: Some("en".to_string()),
            bbox_norm: BBoxNorm {
                x: (b.x * 10_000.0).round() as u32,
                y: (b.y * 10_000.0).round() as u32,
                w: (b.w * 10_000.0).round() as u32,
                h: (b.h * 10_000.0).round() as u32,
            },
        });
    }

    Ok(out)
}

fn is_norm(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}

fn helper_script_path(name: &str) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.join("src/platform/macos/helpers").join(name)
}

#[cfg(test)]
mod tests {
    use super::parse_provider_blocks;

    #[test]
    fn rejects_invalid_provider_schema() {
        let bad = r#"[{"text":"ok","confidence":1.2,"x":0.1,"y":0.1,"w":0.2,"h":0.2}]"#;
        let err = parse_provider_blocks(bad.as_bytes()).expect_err("must fail");
        assert!(err.to_string().contains("provider schema invalid"));
    }
}
