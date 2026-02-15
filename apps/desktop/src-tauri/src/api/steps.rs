use crate::api::Backend;
use crate::steps::derive::StepsCandidatesGeneratedPayload;
use crate::steps::{edit_ops, validate};
use crate::storage::{repo_jobs, repo_ocr, repo_sessions};
use opscinema_types::{
    AppError, AppErrorCode, AppResult, JobHandle, Step, StepDetail, StepId, StepModel,
    StepsApplyEditRequest, StepsApplyEditResponse, StepsGenerateCandidatesRequest, StepsGetRequest,
    StepsListRequest, StepsListResponse, StepsValidateExportResponse, StepsValidateRequest,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct StepEditAppliedPayload {
    base_seq: i64,
    op: opscinema_types::StepEditOp,
    applied_at: String,
}

pub fn steps_generate_candidates(
    backend: &Backend,
    req: StepsGenerateCandidatesRequest,
) -> AppResult<JobHandle> {
    let deterministic = std::env::var("OPSCINEMA_DETERMINISTIC_IDS")
        .map(|v| v == "1")
        .unwrap_or(false);
    let step_id = if deterministic {
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("{}:step:0", req.session_id).as_bytes(),
        )
    } else {
        Uuid::new_v4()
    };
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let mut narrative_text = "Navigate to the target screen".to_string();
    let mut title = "Open target screen".to_string();
    let mut evidence_id = crate::util::ids::deterministic_evidence_id(
        req.session_id,
        "GeneratedStepBlock",
        &format!("{step_id}:b1"),
    );
    if let Some((block, _frame_ms)) = repo_ocr::list_blocks_by_session(&conn, req.session_id)
        .map_err(internal_anyhow)?
        .into_iter()
        .next()
    {
        let snippet = block.text.chars().take(72).collect::<String>();
        if !snippet.trim().is_empty() {
            narrative_text = format!(
                "Open the target screen and verify the visible text contains: \"{}\"",
                snippet
            );
            title = "Open screen and verify key text".to_string();
            evidence_id = crate::util::ids::deterministic_evidence_id(
                req.session_id,
                "OcrSpan",
                &block.ocr_block_id,
            );
        }
    }
    let step = Step {
        step_id,
        order_index: 0,
        title,
        body: opscinema_types::StructuredText {
            blocks: vec![opscinema_types::TextBlock {
                block_id: "b1".to_string(),
                text: narrative_text,
                provenance: opscinema_types::TextBlockProvenance::Generated,
                evidence_refs: vec![evidence_id],
            }],
        },
        risk_tags: vec![],
        branch_label: None,
    };

    let payload = StepsCandidatesGeneratedPayload {
        schema_version: 1,
        steps: vec![step],
    };
    let job_id = repo_jobs::create_job(&conn, "steps_generate_candidates", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        None,
        None,
    );
    let run = crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "StepsCandidatesGenerated",
        &payload,
        None,
    )
    .map_err(internal_anyhow);
    match run {
        Ok(_) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Succeeded,
                None,
                None,
            );
            Ok(JobHandle { job_id })
        }
        Err(err) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                None,
                Some(err.clone()),
            );
            Err(err)
        }
    }
}

pub fn steps_list(backend: &Backend, req: StepsListRequest) -> AppResult<StepsListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let steps = crate::steps::replay::replay_session_steps(&conn, req.session_id)
        .map_err(internal_anyhow)?;
    let head_seq = repo_sessions::get_head_seq(&conn, req.session_id).map_err(internal_anyhow)?;
    Ok(StepsListResponse { steps, head_seq })
}

pub fn steps_get(backend: &Backend, req: StepsGetRequest) -> AppResult<StepDetail> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let step = steps
        .into_iter()
        .find(|s| s.step_id == req.step_id)
        .ok_or_else(|| not_found("step not found"))?;
    let anchors = crate::anchors::cache::list_for_step(&conn, req.session_id, req.step_id)
        .map_err(internal_anyhow)?;
    Ok(StepDetail { step, anchors })
}

pub fn steps_apply_edit(
    backend: &Backend,
    req: StepsApplyEditRequest,
) -> AppResult<StepsApplyEditResponse> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let head_seq = repo_sessions::get_head_seq(&conn, req.session_id).map_err(internal_anyhow)?;
    if req.base_seq != head_seq {
        return Err(AppError {
            code: AppErrorCode::Conflict,
            message: "base sequence does not match session head".to_string(),
            details: Some(format!("expected={} got={}", head_seq, req.base_seq)),
            recoverable: true,
            action_hint: Some("Refetch steps and retry".to_string()),
        });
    }

    let mut replayed = crate::steps::replay::replay_session_steps(&conn, req.session_id)
        .map_err(internal_anyhow)?;
    edit_ops::apply_edit(&mut replayed, &req.op)?;

    let (_event_id, next_seq, _hash) = crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "StepEditApplied",
        &StepEditAppliedPayload {
            base_seq: req.base_seq,
            op: req.op,
            applied_at: chrono::Utc::now().to_rfc3339(),
        },
        None,
    )
    .map_err(internal_anyhow)?;

    Ok(StepsApplyEditResponse {
        head_seq: next_seq,
        applied: true,
    })
}

pub fn steps_validate(
    backend: &Backend,
    req: StepsValidateRequest,
) -> AppResult<StepsValidateExportResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let steps = crate::steps::replay::replay_session_steps(&conn, req.session_id)
        .map_err(internal_anyhow)?;
    let model = StepModel {
        schema_version: 1,
        steps,
    };
    Ok(validate::validate_for_export(&model))
}

#[allow(dead_code)]
fn _keep_type(_id: StepId) {}

fn db_err<E: std::fmt::Display>(e: E) -> AppError {
    AppError {
        code: AppErrorCode::Db,
        message: "database error".to_string(),
        details: Some(e.to_string()),
        recoverable: false,
        action_hint: None,
    }
}

fn not_found(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::NotFound,
        message: msg.to_string(),
        details: None,
        recoverable: true,
        action_hint: None,
    }
}

fn internal(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: msg.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}

fn internal_anyhow(e: anyhow::Error) -> AppError {
    internal(&e.to_string())
}
