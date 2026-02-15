use crate::storage::DbConn;
use opscinema_types::{BBoxNorm, OcrBlock, OcrSearchResponse, OcrStatus};
use uuid::Uuid;

pub fn upsert_blocks(
    conn: &DbConn,
    session_id: uuid::Uuid,
    frame_event_id: uuid::Uuid,
    frame_ms: i64,
    blocks: &[OcrBlock],
) -> anyhow::Result<()> {
    for block in blocks {
        conn.execute(
            "INSERT OR REPLACE INTO ocr_blocks(session_id, frame_event_id, ocr_block_id, frame_ms, text, bbox_json, confidence, language)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                session_id.to_string(),
                frame_event_id.to_string(),
                block.ocr_block_id,
                frame_ms,
                block.text,
                serde_json::to_string(&block.bbox_norm)?,
                block.confidence,
                block.language
            ],
        )?;
    }
    Ok(())
}

pub fn list_blocks_by_session(
    conn: &DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<(OcrBlock, i64)>> {
    let mut stmt = conn.prepare(
        "SELECT ocr_block_id, text, bbox_json, confidence, language, frame_ms FROM ocr_blocks WHERE session_id=?1",
    )?;
    let rows = stmt
        .query_map(rusqlite::params![session_id.to_string()], |r| {
            Ok((
                OcrBlock {
                    ocr_block_id: r.get(0)?,
                    text: r.get(1)?,
                    bbox_norm: serde_json::from_str(&r.get::<_, String>(2)?).unwrap_or(BBoxNorm {
                        x: 0,
                        y: 0,
                        w: 0,
                        h: 0,
                    }),
                    confidence: r.get(3)?,
                    language: r.get(4)?,
                },
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_blocks_for_frame(
    conn: &DbConn,
    session_id: Uuid,
    frame_event_id: Uuid,
) -> anyhow::Result<Vec<OcrBlock>> {
    let mut stmt = conn.prepare(
        "SELECT ocr_block_id, text, bbox_json, confidence, language FROM ocr_blocks WHERE session_id=?1 AND frame_event_id=?2",
    )?;
    let rows = stmt
        .query_map(
            rusqlite::params![session_id.to_string(), frame_event_id.to_string()],
            |r| {
                Ok(OcrBlock {
                    ocr_block_id: r.get(0)?,
                    text: r.get(1)?,
                    bbox_norm: serde_json::from_str(&r.get::<_, String>(2)?).unwrap_or(BBoxNorm {
                        x: 0,
                        y: 0,
                        w: 0,
                        h: 0,
                    }),
                    confidence: r.get(3)?,
                    language: r.get(4)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn status(conn: &DbConn, session_id: Uuid) -> anyhow::Result<OcrStatus> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM ocr_blocks WHERE session_id=?1",
        rusqlite::params![session_id.to_string()],
        |r| r.get::<_, i64>(0),
    )?;
    Ok(OcrStatus {
        queued_frames: 0,
        indexed_frames: count as u32,
    })
}

pub fn search(conn: &DbConn, session_id: Uuid, query: &str) -> anyhow::Result<OcrSearchResponse> {
    let q = query.to_lowercase();
    let hits = list_blocks_by_session(conn, session_id)?
        .into_iter()
        .filter(|(block, _)| block.text.to_lowercase().contains(&q))
        .map(|(block, frame_ms)| opscinema_types::OcrSearchHit {
            frame_ms,
            block_id: block.ocr_block_id,
            snippet: block.text,
        })
        .collect();
    Ok(OcrSearchResponse { hits })
}
