use crate::anchors::drift::detect_drift;
use crate::anchors::providers::vision::VisionAnchorProvider;
use crate::anchors::types::{mark_degraded, AnchorDegradedPayload, AnchorResolvedPayload};
use opscinema_types::AnchorCandidate;
use uuid::Uuid;

pub fn reacquire_anchor(
    provider: &dyn VisionAnchorProvider,
    anchor: &mut AnchorCandidate,
    keyframe_png: &[u8],
) -> anyhow::Result<Result<AnchorResolvedPayload, AnchorDegradedPayload>> {
    let locators = provider.resolve(anchor, keyframe_png)?;
    if detect_drift(&anchor.locators, &locators) {
        anchor.locators = locators.clone();
        let resolved = AnchorResolvedPayload {
            anchor_id: anchor.anchor_id,
            resolved_locators: locators,
            confidence: anchor.confidence,
            provenance: "vision_keyframe".to_string(),
            supporting_evidence_ids: vec![Uuid::new_v4()],
            provider_output_asset_id: None,
        };
        Ok(Ok(resolved))
    } else {
        Ok(Err(mark_degraded(anchor, "NO_MATCH")))
    }
}
