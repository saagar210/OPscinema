use crate::storage::event_store;
use opscinema_types::{TimelineEvent, TimelineKeyframe};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct KeyframeCaptured {
    frame_ms: i64,
    asset_id: String,
}

pub fn get_events(
    conn: &rusqlite::Connection,
    session_id: Uuid,
    after_seq: Option<i64>,
    limit: u32,
) -> anyhow::Result<Vec<TimelineEvent>> {
    let rows = event_store::query_events(conn, session_id, after_seq, limit)?;
    Ok(rows
        .into_iter()
        .map(|r| TimelineEvent {
            seq: r.seq,
            event_id: Uuid::parse_str(&r.event_id).unwrap_or_else(|_| Uuid::nil()),
            event_type: r.event_type,
            frame_ms: serde_json::from_str::<serde_json::Value>(&r.payload_canon_json)
                .ok()
                .and_then(|v| v.get("frame_ms").and_then(|n| n.as_i64())),
        })
        .collect())
}

pub fn get_keyframes(
    conn: &rusqlite::Connection,
    session_id: Uuid,
    start_ms: i64,
    end_ms: i64,
) -> anyhow::Result<Vec<TimelineKeyframe>> {
    let rows = event_store::query_events(conn, session_id, None, 10_000)?;
    let mut keyframes = Vec::new();
    for row in rows {
        if row.event_type != "KeyframeCaptured" {
            continue;
        }
        if let Ok(k) = serde_json::from_str::<KeyframeCaptured>(&row.payload_canon_json) {
            if (start_ms..=end_ms).contains(&k.frame_ms) {
                keyframes.push(TimelineKeyframe {
                    frame_ms: k.frame_ms,
                    frame_event_id: Uuid::parse_str(&row.event_id).unwrap_or_else(|_| Uuid::nil()),
                    asset: opscinema_types::AssetRef {
                        asset_id: k.asset_id,
                    },
                });
            }
        }
    }
    Ok(keyframes)
}

pub fn get_thumbnail_asset(
    conn: &rusqlite::Connection,
    session_id: Uuid,
    frame_event_id: Uuid,
) -> anyhow::Result<Option<String>> {
    let rows = event_store::query_events(conn, session_id, None, 10_000)?;
    for row in rows {
        if row.event_id != frame_event_id.to_string() || row.event_type != "KeyframeCaptured" {
            continue;
        }
        if let Ok(k) = serde_json::from_str::<KeyframeCaptured>(&row.payload_canon_json) {
            return Ok(Some(k.asset_id));
        }
    }
    Ok(None)
}
