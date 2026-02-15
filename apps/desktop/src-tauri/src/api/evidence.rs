use crate::api::Backend;
use crate::evidence::coverage;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, EvidenceCoverageRequest, EvidenceCoverageResponse,
    EvidenceFindTextRequest, EvidenceFindTextResponse, EvidenceForStepRequest,
    EvidenceForTimeRangeRequest, EvidenceSet,
};

pub fn evidence_for_time_range(
    backend: &Backend,
    req: EvidenceForTimeRangeRequest,
) -> AppResult<EvidenceSet> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let set = crate::evidence::graph::derive_from_event_log(&conn, req.session_id)
        .map_err(internal_anyhow)?;

    let evidence = set
        .evidence
        .into_iter()
        .filter(|item| {
            item.locators
                .iter()
                .filter_map(|l| l.frame_ms)
                .any(|ms| ms >= req.start_ms && ms <= req.end_ms)
                || item.locators.iter().all(|l| l.frame_ms.is_none())
        })
        .collect();
    Ok(EvidenceSet { evidence })
}

pub fn evidence_for_step(backend: &Backend, req: EvidenceForStepRequest) -> AppResult<EvidenceSet> {
    let steps = crate::api::steps::steps_list(
        backend,
        opscinema_types::StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let step = steps
        .iter()
        .find(|s| s.step_id == req.step_id)
        .ok_or_else(|| not_found("step not found"))?
        .clone();
    let all = evidence_for_time_range(
        backend,
        EvidenceForTimeRangeRequest {
            session_id: req.session_id,
            start_ms: 0,
            end_ms: i64::MAX,
        },
    )?;
    Ok(crate::evidence::query::for_step(&step, &all))
}

pub fn evidence_find_text(
    backend: &Backend,
    req: EvidenceFindTextRequest,
) -> AppResult<EvidenceFindTextResponse> {
    let all = evidence_for_time_range(
        backend,
        EvidenceForTimeRangeRequest {
            session_id: req.session_id,
            start_ms: 0,
            end_ms: i64::MAX,
        },
    )?;
    let q = req.query.to_lowercase();
    let evidence = all
        .evidence
        .into_iter()
        .filter(|e| {
            e.locators.iter().any(|l| {
                l.note
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&q))
                    .unwrap_or(false)
            })
        })
        .collect();
    Ok(EvidenceFindTextResponse { evidence })
}

pub fn evidence_get_coverage(
    backend: &Backend,
    req: EvidenceCoverageRequest,
) -> AppResult<EvidenceCoverageResponse> {
    let steps = crate::api::steps::steps_list(
        backend,
        opscinema_types::StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    Ok(coverage::evaluate(&steps))
}

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
