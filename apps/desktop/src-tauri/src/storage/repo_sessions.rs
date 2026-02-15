use crate::util::hash::blake3_hex;
use crate::util::time::now_utc_iso;
use opscinema_types::{SessionDetail, SessionSummary};
use rusqlite::{params, OptionalExtension};
use std::collections::BTreeMap;
use uuid::Uuid;

pub fn create_session(conn: &rusqlite::Connection, label: &str) -> anyhow::Result<SessionSummary> {
    let deterministic = std::env::var("OPSCINEMA_DETERMINISTIC_IDS")
        .map(|v| v == "1")
        .unwrap_or(false);
    let session_id = if deterministic {
        Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("session:{label}").as_bytes())
    } else {
        Uuid::new_v4()
    };
    let created_at = now_utc_iso();
    let head_hash = blake3_hex(format!("{}:{}", session_id, label).as_bytes());

    conn.execute(
        "INSERT INTO sessions(session_id,label,created_at,head_seq,head_hash) VALUES (?1, ?2, ?3, 0, ?4)",
        params![session_id.to_string(), label, created_at, head_hash],
    )?;

    Ok(SessionSummary {
        session_id,
        label: label.to_string(),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?.with_timezone(&chrono::Utc),
        closed_at: None,
        head_seq: 0,
        head_hash,
    })
}

pub fn list_sessions(
    conn: &rusqlite::Connection,
    limit: u32,
) -> anyhow::Result<Vec<SessionSummary>> {
    let mut stmt = conn.prepare(
        "SELECT session_id,label,created_at,closed_at,head_seq,head_hash FROM sessions ORDER BY created_at DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit as i64], |r| {
            let created_raw: String = r.get(2)?;
            let closed_raw: Option<String> = r.get(3)?;
            Ok(SessionSummary {
                session_id: Uuid::parse_str(&r.get::<_, String>(0)?)
                    .unwrap_or_else(|_| Uuid::nil()),
                label: r.get(1)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_raw)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                closed_at: closed_raw.and_then(|v| {
                    chrono::DateTime::parse_from_rfc3339(&v)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                head_seq: r.get(4)?,
                head_hash: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_session(
    conn: &rusqlite::Connection,
    session_id: Uuid,
) -> anyhow::Result<Option<SessionDetail>> {
    let summary = conn
        .query_row(
            "SELECT session_id,label,created_at,closed_at,head_seq,head_hash FROM sessions WHERE session_id=?1",
            params![session_id.to_string()],
            |r| {
                let created_raw: String = r.get(2)?;
                let closed_raw: Option<String> = r.get(3)?;
                Ok(SessionSummary {
                    session_id: Uuid::parse_str(&r.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
                    label: r.get(1)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_raw)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    closed_at: closed_raw.and_then(|v| {
                        chrono::DateTime::parse_from_rfc3339(&v)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    head_seq: r.get(4)?,
                    head_hash: r.get(5)?,
                })
            },
        )
        .optional()?;

    Ok(summary.map(|summary| SessionDetail {
        summary,
        metadata: BTreeMap::new(),
    }))
}

pub fn close_session(conn: &rusqlite::Connection, session_id: Uuid) -> anyhow::Result<bool> {
    let closed_at = now_utc_iso();
    let affected = conn.execute(
        "UPDATE sessions SET closed_at=?2 WHERE session_id=?1",
        params![session_id.to_string(), closed_at],
    )?;
    Ok(affected > 0)
}

pub fn get_head_seq(conn: &rusqlite::Connection, session_id: Uuid) -> anyhow::Result<i64> {
    Ok(conn
        .query_row(
            "SELECT head_seq FROM sessions WHERE session_id=?1",
            params![session_id.to_string()],
            |r| r.get(0),
        )
        .unwrap_or(0))
}
