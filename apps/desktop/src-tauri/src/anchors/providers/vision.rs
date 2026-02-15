use opscinema_types::{AnchorCandidate, EvidenceLocator};

pub trait VisionAnchorProvider: Send + Sync {
    fn resolve(
        &self,
        anchor: &AnchorCandidate,
        keyframe_png: &[u8],
    ) -> anyhow::Result<Vec<EvidenceLocator>>;
}

#[derive(Default)]
pub struct StubVisionAnchorProvider;

impl VisionAnchorProvider for StubVisionAnchorProvider {
    fn resolve(
        &self,
        anchor: &AnchorCandidate,
        _keyframe_png: &[u8],
    ) -> anyhow::Result<Vec<EvidenceLocator>> {
        Ok(anchor.locators.clone())
    }
}
