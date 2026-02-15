use crate::util::canon_json::to_canonical_json;
use crate::util::hash::blake3_hex;
use crate::util::time::now_utc_iso;
use anyhow::Context;
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EventRow {
    pub session_id: String,
    pub seq: i64,
    pub event_id: String,
    pub event_type: String,
    pub payload_canon_json: String,
    pub event_hash: String,
}

#[derive(Debug, Clone)]
pub enum CrashPoint {
    AfterEventInsertBeforeCommit,
}

pub fn append_event<T: Serialize>(
    conn: &mut rusqlite::Connection,
    session_id: Uuid,
    event_type: &str,
    payload: &T,
    crash: Option<CrashPoint>,
) -> anyhow::Result<(Uuid, i64, String)> {
    let tx = conn.transaction()?;

    let (head_seq, head_hash): (i64, String) = tx
        .query_row(
            "SELECT head_seq, head_hash FROM sessions WHERE session_id=?1",
            params![session_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?
        .unwrap_or((0, String::new()));

    let seq = head_seq + 1;
    let event_id = Uuid::new_v4();
    let payload_canon_json = to_canonical_json(payload).context("canonicalize payload")?;
    let prev = if head_hash.is_empty() {
        "GENESIS".to_string()
    } else {
        head_hash.clone()
    };
    let hash_input = format!(
        "{}\n{}\n{}\n{}\n{}\n",
        session_id, seq, event_type, payload_canon_json, prev
    );
    let event_hash = blake3_hex(hash_input.as_bytes());

    tx.execute(
        "INSERT INTO events(session_id, seq, event_id, event_type, payload_canon_json, prev_event_hash, event_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session_id.to_string(),
            seq,
            event_id.to_string(),
            event_type,
            payload_canon_json,
            if head_hash.is_empty() { None::<String> } else { Some(head_hash.clone()) },
            event_hash,
            now_utc_iso(),
        ],
    )?;

    if matches!(crash, Some(CrashPoint::AfterEventInsertBeforeCommit)) {
        anyhow::bail!("simulated crash after event insert")
    }

    tx.execute(
        "INSERT INTO sessions(session_id,label,created_at,closed_at,head_seq,head_hash)
         VALUES (?1, ?2, ?3, NULL, ?4, ?5)
         ON CONFLICT(session_id) DO UPDATE SET head_seq=excluded.head_seq, head_hash=excluded.head_hash",
        params![session_id.to_string(), "session", now_utc_iso(), seq, event_hash],
    )?;

    tx.commit()?;
    Ok((event_id, seq, event_hash))
}

pub fn query_events(
    conn: &rusqlite::Connection,
    session_id: Uuid,
    after_seq: Option<i64>,
    limit: u32,
) -> anyhow::Result<Vec<EventRow>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, seq, event_id, event_type, payload_canon_json, event_hash
         FROM events
         WHERE session_id=?1 AND seq>?2
         ORDER BY seq ASC
         LIMIT ?3",
    )?;
    let rows = stmt
        .query_map(
            params![session_id.to_string(), after_seq.unwrap_or(0), limit as i64],
            |r| {
                Ok(EventRow {
                    session_id: r.get(0)?,
                    seq: r.get(1)?,
                    event_id: r.get(2)?,
                    event_type: r.get(3)?,
                    payload_canon_json: r.get(4)?,
                    event_hash: r.get(5)?,
                })
            },
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn validate_hash_chain(conn: &rusqlite::Connection, session_id: Uuid) -> anyhow::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT seq, event_type, payload_canon_json, prev_event_hash, event_hash
         FROM events
         WHERE session_id=?1
         ORDER BY seq ASC",
    )?;
    let rows = stmt.query_map(params![session_id.to_string()], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;

    let mut expected_prev: Option<String> = None;
    let mut last_seq = 0_i64;
    let mut last_hash = String::new();
    for row in rows {
        let (seq, event_type, payload_canon_json, prev_event_hash, event_hash) = row?;
        let prev = expected_prev
            .clone()
            .unwrap_or_else(|| "GENESIS".to_string());
        let hash_input = format!(
            "{}\n{}\n{}\n{}\n{}\n",
            session_id, seq, event_type, payload_canon_json, prev
        );
        let recomputed = blake3_hex(hash_input.as_bytes());
        if recomputed != event_hash {
            anyhow::bail!("event hash mismatch at seq {seq}")
        }

        if expected_prev.is_none() {
            if prev_event_hash.is_some() {
                anyhow::bail!("genesis event has unexpected prev hash")
            }
        } else if prev_event_hash != expected_prev {
            anyhow::bail!("prev_event_hash mismatch at seq {seq}")
        }

        expected_prev = Some(event_hash.clone());
        last_seq = seq;
        last_hash = event_hash;
    }

    let head: Option<(i64, String)> = conn
        .query_row(
            "SELECT head_seq, head_hash FROM sessions WHERE session_id=?1",
            params![session_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((head_seq, head_hash)) = head {
        if head_seq != last_seq || (!head_hash.is_empty() && head_hash != last_hash) {
            anyhow::bail!("session head does not match validated chain")
        }
    }
    Ok(())
}
