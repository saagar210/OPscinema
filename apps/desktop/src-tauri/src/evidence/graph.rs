use crate::util::ids::deterministic_evidence_id;
use opscinema_types::{EvidenceItem, EvidenceLocator, EvidenceLocatorType, EvidenceSet, OcrBlock};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct KeyframeCaptured {
    frame_ms: i64,
    asset_id: String,
}

#[derive(Debug, Deserialize)]
struct ClickCaptured {
    frame_ms: i64,
    display_id: String,
    pos_norm: ClickPosNorm,
}

#[derive(Debug, Deserialize)]
struct ClickPosNorm {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize)]
struct WindowMetaCaptured {
    frame_ms: i64,
    frontmost_bundle_id: Option<String>,
    frontmost_title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OcrBlocksPersisted {
    frame_ms: i64,
    ocr_asset_id: String,
    provider_output_asset_id: Option<String>,
    blocks: Vec<OcrBlock>,
}

#[derive(Debug, Deserialize)]
struct VerifierRunCompleted {
    run_id: Uuid,
    result_asset_id: String,
}

#[derive(Debug, Deserialize)]
struct AnchorResolved {
    anchor_id: Uuid,
    resolved_locators: Vec<EvidenceLocator>,
    provider_output_asset_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnchorDegraded {
    anchor_id: Uuid,
    reason_code: String,
    last_verified_locators: Vec<EvidenceLocator>,
}

#[derive(Debug, Deserialize)]
struct ExportCreated {
    export_id: Uuid,
    output_path: String,
    manifest_asset_id: String,
}

pub fn derive_from_event_log(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<EvidenceSet> {
    let events = crate::storage::event_store::query_events(conn, session_id, None, 100_000)?;
    let mut evidence = Vec::new();

    for event in events {
        match event.event_type.as_str() {
            "KeyframeCaptured" => {
                let payload: KeyframeCaptured = serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "FrameKeyframe",
                        &event.event_id,
                    ),
                    kind: "FrameKeyframe".to_string(),
                    source_id: event.event_id,
                    locators: vec![EvidenceLocator {
                        locator_type: EvidenceLocatorType::FrameBbox,
                        asset_id: Some(payload.asset_id),
                        frame_ms: Some(payload.frame_ms),
                        bbox_norm: None,
                        text_offset: None,
                        note: None,
                    }],
                });
            }
            "ClickCaptured" => {
                let payload: ClickCaptured = serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(session_id, "Click", &event.event_id),
                    kind: "Click".to_string(),
                    source_id: event.event_id.clone(),
                    locators: vec![EvidenceLocator {
                        locator_type: EvidenceLocatorType::Timeline,
                        asset_id: None,
                        frame_ms: Some(payload.frame_ms),
                        bbox_norm: Some(opscinema_types::BBoxNorm {
                            x: (payload.pos_norm.x.clamp(0.0, 1.0) * 10_000.0).round() as u32,
                            y: (payload.pos_norm.y.clamp(0.0, 1.0) * 10_000.0).round() as u32,
                            w: 1,
                            h: 1,
                        }),
                        text_offset: None,
                        note: Some(format!("display={}", payload.display_id)),
                    }],
                });
            }
            "WindowMetaCaptured" => {
                let payload: WindowMetaCaptured = serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "WindowMeta",
                        &event.event_id,
                    ),
                    kind: "WindowMeta".to_string(),
                    source_id: event.event_id.clone(),
                    locators: vec![EvidenceLocator {
                        locator_type: EvidenceLocatorType::Timeline,
                        asset_id: None,
                        frame_ms: Some(payload.frame_ms),
                        bbox_norm: None,
                        text_offset: None,
                        note: Some(format!(
                            "{}:{}",
                            payload.frontmost_bundle_id.unwrap_or_default(),
                            payload.frontmost_title.unwrap_or_default()
                        )),
                    }],
                });
            }
            "OcrBlocksPersisted" => {
                let payload: OcrBlocksPersisted = serde_json::from_str(&event.payload_canon_json)?;
                for block in payload.blocks {
                    evidence.push(EvidenceItem {
                        evidence_id: deterministic_evidence_id(
                            session_id,
                            "OcrSpan",
                            &block.ocr_block_id,
                        ),
                        kind: "OcrSpan".to_string(),
                        source_id: block.ocr_block_id,
                        locators: vec![EvidenceLocator {
                            locator_type: EvidenceLocatorType::OcrBbox,
                            asset_id: Some(payload.ocr_asset_id.clone()),
                            frame_ms: Some(payload.frame_ms),
                            bbox_norm: Some(block.bbox_norm),
                            text_offset: None,
                            note: Some(block.text),
                        }],
                    });
                }
                if let Some(provider_asset_id) = payload.provider_output_asset_id {
                    evidence.push(EvidenceItem {
                        evidence_id: deterministic_evidence_id(
                            session_id,
                            "OcrProviderOutput",
                            &provider_asset_id,
                        ),
                        kind: "OcrProviderOutput".to_string(),
                        source_id: provider_asset_id.clone(),
                        locators: vec![EvidenceLocator {
                            locator_type: EvidenceLocatorType::FilePath,
                            asset_id: Some(provider_asset_id),
                            frame_ms: Some(payload.frame_ms),
                            bbox_norm: None,
                            text_offset: None,
                            note: Some("ocr provider output".to_string()),
                        }],
                    });
                }
            }
            "VerifierRunCompleted" => {
                let payload: VerifierRunCompleted =
                    serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "VerifierResult",
                        &payload.run_id.to_string(),
                    ),
                    kind: "VerifierResult".to_string(),
                    source_id: payload.run_id.to_string(),
                    locators: vec![EvidenceLocator {
                        locator_type: EvidenceLocatorType::VerifierLog,
                        asset_id: Some(payload.result_asset_id),
                        frame_ms: None,
                        bbox_norm: None,
                        text_offset: None,
                        note: None,
                    }],
                });
            }
            "AnchorResolved" => {
                let payload: AnchorResolved = serde_json::from_str(&event.payload_canon_json)?;
                let mut locators = payload.resolved_locators;
                if let Some(asset_id) = payload.provider_output_asset_id {
                    locators.push(EvidenceLocator {
                        locator_type: EvidenceLocatorType::FilePath,
                        asset_id: Some(asset_id),
                        frame_ms: None,
                        bbox_norm: None,
                        text_offset: None,
                        note: Some("anchor provider output".to_string()),
                    });
                }
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "AnchorObservation",
                        &payload.anchor_id.to_string(),
                    ),
                    kind: "AnchorObservation".to_string(),
                    source_id: payload.anchor_id.to_string(),
                    locators,
                });
            }
            "AnchorDegraded" => {
                let payload: AnchorDegraded = serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "AnchorDegraded",
                        &payload.anchor_id.to_string(),
                    ),
                    kind: "AnchorDegraded".to_string(),
                    source_id: payload.anchor_id.to_string(),
                    locators: payload
                        .last_verified_locators
                        .into_iter()
                        .map(|mut locator| {
                            locator.note = Some(format!("degraded:{}", payload.reason_code));
                            locator
                        })
                        .collect(),
                });
            }
            "ExportCreated" => {
                let payload: ExportCreated = serde_json::from_str(&event.payload_canon_json)?;
                evidence.push(EvidenceItem {
                    evidence_id: deterministic_evidence_id(
                        session_id,
                        "ExportBundle",
                        &payload.export_id.to_string(),
                    ),
                    kind: "ExportBundle".to_string(),
                    source_id: payload.export_id.to_string(),
                    locators: vec![EvidenceLocator {
                        locator_type: EvidenceLocatorType::FilePath,
                        asset_id: Some(payload.manifest_asset_id),
                        frame_ms: None,
                        bbox_norm: None,
                        text_offset: None,
                        note: Some(payload.output_path),
                    }],
                });
            }
            _ => {}
        }
    }

    Ok(EvidenceSet { evidence })
}
