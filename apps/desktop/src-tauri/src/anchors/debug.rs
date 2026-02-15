use opscinema_types::{AnchorCandidate, AnchorsDebugResponse};

pub fn debug_anchor(anchor: &AnchorCandidate) -> AnchorsDebugResponse {
    AnchorsDebugResponse {
        checks: vec![
            format!("anchor_id={}", anchor.anchor_id),
            format!("confidence={}", anchor.confidence),
            format!("degraded={}", anchor.degraded),
        ],
    }
}
