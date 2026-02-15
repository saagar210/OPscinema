use crate::capture::screen::ScreenKeyframe;
use crate::ocr::vision::VisionProvider;
use crate::storage::asset_store::AssetStore;
use crate::storage::event_store::append_event;
use crate::storage::repo_ocr;
use crate::storage::DbConn;
use opscinema_types::OcrBlock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrBlocksPersistedPayload {
    pub frame_event_id: Uuid,
    pub frame_ms: i64,
    pub ocr_asset_id: String,
    pub provider_output_asset_id: String,
    pub blocks: Vec<OcrBlock>,
}

pub fn persist_ocr_for_frame(
    conn: &mut DbConn,
    store: &AssetStore,
    provider: &dyn VisionProvider,
    session_id: Uuid,
    frame_event_id: Uuid,
    keyframe: &ScreenKeyframe,
) -> anyhow::Result<Vec<OcrBlock>> {
    let blocks = provider.recognize(&keyframe.png_bytes)?;
    let blocks_json = crate::util::canon_json::to_canonical_json(&blocks)?;
    let asset_id = store.put(conn, blocks_json.as_bytes(), None)?;
    let provider_raw = std::env::var("OPSCINEMA_VISION_RAW_JSON")
        .ok()
        .and_then(|raw| {
            serde_json::from_str::<serde_json::Value>(&raw)
                .ok()
                .and_then(|v| crate::util::canon_json::to_canonical_json(&v).ok())
        })
        .unwrap_or_else(|| blocks_json.clone());
    let provider_output_asset_id = store.put(conn, provider_raw.as_bytes(), None)?;

    repo_ocr::upsert_blocks(conn, session_id, frame_event_id, keyframe.frame_ms, &blocks)?;

    let payload = OcrBlocksPersistedPayload {
        frame_event_id,
        frame_ms: keyframe.frame_ms,
        ocr_asset_id: asset_id,
        provider_output_asset_id,
        blocks: blocks.clone(),
    };
    append_event(conn, session_id, "OcrBlocksPersisted", &payload, None)?;

    Ok(blocks)
}
