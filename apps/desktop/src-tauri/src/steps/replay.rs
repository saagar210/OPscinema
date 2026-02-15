use crate::steps::derive::StepsCandidatesGeneratedPayload;
use crate::steps::edit_ops::apply_edit;
use opscinema_types::{Step, StepEditOp};
use serde::Deserialize;
use uuid::Uuid;

pub fn replay_from_event_payloads(initial: &str, edits: &[String]) -> anyhow::Result<Vec<Step>> {
    let init: StepsCandidatesGeneratedPayload = serde_json::from_str(initial)?;
    let mut steps = init.steps;
    for raw in edits {
        let op: StepEditOp = serde_json::from_str(raw)?;
        apply_edit(&mut steps, &op)?;
    }
    Ok(steps)
}

#[derive(Debug, Deserialize)]
struct StepEditAppliedPayload {
    op: StepEditOp,
}

pub fn replay_session_steps(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<Step>> {
    let events = crate::storage::event_store::query_events(conn, session_id, None, 100_000)?;
    let mut steps = Vec::new();
    let mut has_initial = false;

    for row in events {
        match row.event_type.as_str() {
            "StepsCandidatesGenerated" => {
                let payload: StepsCandidatesGeneratedPayload =
                    serde_json::from_str(&row.payload_canon_json)?;
                steps = payload.steps;
                has_initial = true;
            }
            "StepEditApplied" => {
                if !has_initial {
                    continue;
                }
                let payload: StepEditAppliedPayload =
                    serde_json::from_str(&row.payload_canon_json)?;
                apply_edit(&mut steps, &payload.op)?;
            }
            _ => {}
        }
    }

    Ok(steps)
}
