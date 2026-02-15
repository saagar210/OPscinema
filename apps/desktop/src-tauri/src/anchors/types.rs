use opscinema_types::{AnchorCandidate, AnchorId, EvidenceLocator};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorResolvedPayload {
    pub anchor_id: AnchorId,
    pub resolved_locators: Vec<EvidenceLocator>,
    pub confidence: u8,
    pub provenance: String,
    pub supporting_evidence_ids: Vec<Uuid>,
    pub provider_output_asset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorDegradedPayload {
    pub anchor_id: AnchorId,
    pub reason_code: String,
    pub details: String,
    pub last_verified_locators: Vec<EvidenceLocator>,
    pub degraded_at: String,
}

pub fn mark_degraded(anchor: &mut AnchorCandidate, reason: &str) -> AnchorDegradedPayload {
    anchor.degraded = true;
    AnchorDegradedPayload {
        anchor_id: anchor.anchor_id,
        reason_code: reason.to_string(),
        details: "drift exceeded threshold".to_string(),
        last_verified_locators: anchor.locators.clone(),
        degraded_at: chrono::Utc::now().to_rfc3339(),
    }
}
