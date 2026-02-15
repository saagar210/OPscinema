use opscinema_types::{AnchorCandidate, AnchorId, EvidenceLocator, StepId};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct AnchorCandidatesGeneratedPayload {
    step_id: StepId,
    anchors: Vec<AnchorCandidate>,
}

#[derive(Debug, Deserialize)]
struct AnchorResolvedPayload {
    anchor_id: AnchorId,
    resolved_locators: Vec<EvidenceLocator>,
    confidence: u8,
}

#[derive(Debug, Deserialize)]
struct AnchorDegradedPayload {
    anchor_id: AnchorId,
    last_verified_locators: Vec<EvidenceLocator>,
}

#[derive(Debug, Deserialize)]
struct AnchorManuallySetPayload {
    anchor_id: AnchorId,
    locators: Vec<EvidenceLocator>,
    manual_note: Option<String>,
}

pub fn replay_session(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<AnchorCandidate>> {
    let events = crate::storage::event_store::query_events(conn, session_id, None, 100_000)?;
    let mut anchors: BTreeMap<AnchorId, AnchorCandidate> = BTreeMap::new();

    for event in events {
        match event.event_type.as_str() {
            "AnchorCandidatesGenerated" => {
                let payload: AnchorCandidatesGeneratedPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                for mut anchor in payload.anchors {
                    anchor.step_id = payload.step_id;
                    anchors.insert(anchor.anchor_id, anchor);
                }
            }
            "AnchorResolved" => {
                let payload: AnchorResolvedPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                if let Some(anchor) = anchors.get_mut(&payload.anchor_id) {
                    anchor.locators = payload.resolved_locators;
                    anchor.confidence = payload.confidence;
                    anchor.degraded = false;
                }
            }
            "AnchorDegraded" => {
                let payload: AnchorDegradedPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                if let Some(anchor) = anchors.get_mut(&payload.anchor_id) {
                    anchor.locators = payload.last_verified_locators;
                    anchor.degraded = true;
                }
            }
            "AnchorManuallySet" => {
                let payload: AnchorManuallySetPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                if let Some(anchor) = anchors.get_mut(&payload.anchor_id) {
                    anchor.locators = payload.locators;
                    anchor.degraded = false;
                }
                let _ = payload.manual_note;
            }
            _ => {}
        }
    }

    Ok(anchors.into_values().collect())
}

pub fn list_for_step(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
    step_id: StepId,
) -> anyhow::Result<Vec<AnchorCandidate>> {
    let mut anchors = replay_session(conn, session_id)?
        .into_iter()
        .filter(|a| a.step_id == step_id)
        .collect::<Vec<_>>();
    anchors.sort_by(|a, b| a.anchor_id.cmp(&b.anchor_id));
    Ok(anchors)
}
