use crate::api::exports::tutorial_export_pack as export_tutorial_pack;
use crate::api::steps::steps_list;
use crate::api::Backend;
use crate::policy::export_gate::{tutorial_pack_gate, ExportGateInput};
use crate::storage::{repo_jobs, repo_timeline};
use opscinema_types::{
    AppError, AppErrorCode, AppResult, ExplainThisScreenRequest, ExportResult, JobHandle,
    StepsListRequest, TutorialExportRequest, TutorialGenerateRequest,
    TutorialValidateExportRequest, TutorialValidateExportResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct TutorialGeneratedPayload {
    step_count: usize,
    missing_generated_block_ids: Vec<String>,
    narrative_preview: Option<String>,
}

#[derive(Debug, Serialize)]
struct ScreenExplainedPayload {
    frame_event_id: uuid::Uuid,
    summary_asset_id: String,
}

pub fn tutorial_generate(backend: &Backend, req: TutorialGenerateRequest) -> AppResult<JobHandle> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "tutorial_generate", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        None,
        None,
    );

    let run = (|| -> AppResult<()> {
        let before = steps_list(
            backend,
            StepsListRequest {
                session_id: req.session_id,
            },
        )?;
        if before.steps.is_empty() {
            let _ = crate::api::steps::steps_generate_candidates(
                backend,
                opscinema_types::StepsGenerateCandidatesRequest {
                    session_id: req.session_id,
                },
            )?;
        }
        let after = steps_list(
            backend,
            StepsListRequest {
                session_id: req.session_id,
            },
        )?;
        let coverage = crate::evidence::coverage::evaluate(&after.steps);
        crate::storage::event_store::append_event(
            &mut conn,
            req.session_id,
            "TutorialGenerated",
            &TutorialGeneratedPayload {
                step_count: after.steps.len(),
                missing_generated_block_ids: coverage.missing_generated_block_ids,
                narrative_preview: after.steps.first().and_then(|s| {
                    s.body
                        .blocks
                        .first()
                        .map(|b| b.text.chars().take(96).collect::<String>())
                }),
            },
            None,
        )
        .map_err(internal_anyhow)?;
        Ok(())
    })();

    match run {
        Ok(()) => {
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

pub fn tutorial_export_pack(
    backend: &Backend,
    req: TutorialExportRequest,
) -> AppResult<ExportResult> {
    let validate = tutorial_validate_export(
        backend,
        TutorialValidateExportRequest {
            session_id: req.session_id,
        },
    )?;
    if !validate.allowed {
        return Err(AppError {
            code: AppErrorCode::ExportGateFailed,
            message: "Tutorial export blocked by strict gate".to_string(),
            details: Some(validate.reasons.join("; ")),
            recoverable: true,
            action_hint: Some("Resolve strict gate failures and retry".to_string()),
        });
    }
    export_tutorial_pack(backend, req)
}

pub fn tutorial_validate_export(
    backend: &Backend,
    req: TutorialValidateExportRequest,
) -> AppResult<TutorialValidateExportResponse> {
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;

    let coverage = crate::evidence::coverage::evaluate(&steps);
    let conn = backend.storage.conn().map_err(db_err)?;
    let warnings = collect_tutorial_warnings(&conn, req.session_id).map_err(internal_anyhow)?;
    let degraded_anchor_ids = crate::anchors::cache::replay_session(&conn, req.session_id)
        .map_err(internal_anyhow)?
        .into_iter()
        .filter(|a| a.degraded)
        .map(|a| a.anchor_id.to_string())
        .collect::<Vec<_>>();

    let gate = tutorial_pack_gate(&ExportGateInput {
        steps,
        missing_evidence: coverage
            .missing_generated_block_ids
            .iter()
            .map(ToString::to_string)
            .collect(),
        degraded_anchor_ids: degraded_anchor_ids.clone(),
        warnings: warnings.clone(),
    });

    match gate {
        Ok(()) => Ok(TutorialValidateExportResponse {
            allowed: true,
            reasons: vec![],
        }),
        Err(err) => {
            let mut reasons = vec![err.message];
            if !coverage.missing_generated_block_ids.is_empty() {
                reasons.push(format!(
                    "missing evidence refs for generated blocks: {}",
                    coverage.missing_generated_block_ids.join(",")
                ));
            }
            if let Some(details) = err.details {
                reasons.push(details);
            }
            if !degraded_anchor_ids.is_empty() {
                reasons.push(format!(
                    "degraded anchors: {}",
                    degraded_anchor_ids.join(",")
                ));
            }
            if !warnings.is_empty() {
                reasons.push(format!(
                    "warnings present: {}",
                    warnings
                        .iter()
                        .map(|w| format!("{}={}", w.code, w.message))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            Ok(TutorialValidateExportResponse {
                allowed: false,
                reasons,
            })
        }
    }
}

pub fn explain_this_screen(
    backend: &Backend,
    req: ExplainThisScreenRequest,
) -> AppResult<JobHandle> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let asset_id = repo_timeline::get_thumbnail_asset(&conn, req.session_id, req.frame_event_id)
        .map_err(internal_anyhow)?
        .ok_or_else(|| AppError {
            code: AppErrorCode::NotFound,
            message: "frame not found".to_string(),
            details: None,
            recoverable: true,
            action_hint: Some("Select a captured frame".to_string()),
        })?;
    let job_id = repo_jobs::create_job(&conn, "explain_this_screen", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        None,
        None,
    );
    let run = (|| -> AppResult<()> {
        let bytes = std::fs::read(backend.assets.path_for(&asset_id))
            .map_err(|e| internal(&e.to_string()))?;
        let provider = crate::platform::macos::vision_ocr::provider();
        let blocks = provider.recognize(&bytes).map_err(internal_anyhow)?;
        let summary = if let Some(first) = blocks.first() {
            format!("Detected screen text: {}", first.text)
        } else {
            "No OCR text detected in selected frame".to_string()
        };
        let summary_json = crate::util::canon_json::to_canonical_json(
            &serde_json::json!({ "frame_event_id": req.frame_event_id, "summary": summary }),
        )
        .map_err(|e| internal(&e.to_string()))?;
        let summary_asset_id = backend
            .assets
            .put(&conn, summary_json.as_bytes(), None)
            .map_err(internal_anyhow)?;
        crate::storage::event_store::append_event(
            &mut conn,
            req.session_id,
            "ScreenExplained",
            &ScreenExplainedPayload {
                frame_event_id: req.frame_event_id,
                summary_asset_id,
            },
            None,
        )
        .map_err(internal_anyhow)?;
        Ok(())
    })();

    match run {
        Ok(()) => {
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

#[allow(dead_code)]
fn _internal(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: msg.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}

#[derive(Debug, Deserialize)]
struct VerifierRunCompletedPayload {
    verifier_id: String,
    status: String,
}

fn collect_tutorial_warnings(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<opscinema_types::ExportWarning>> {
    let events = crate::storage::event_store::query_events(conn, session_id, None, 100_000)?;
    let mut warnings = Vec::new();
    for event in events {
        if event.event_type != "VerifierRunCompleted" {
            continue;
        }
        let payload: VerifierRunCompletedPayload = serde_json::from_str(&event.payload_canon_json)?;
        if payload.status.eq_ignore_ascii_case("SUCCEEDED") {
            continue;
        }
        warnings.push(opscinema_types::ExportWarning {
            code: "VERIFIER_WARN".to_string(),
            message: format!("Verifier {} status {}", payload.verifier_id, payload.status),
        });
    }
    Ok(warnings)
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

fn internal_anyhow(e: anyhow::Error) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: e.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}

fn internal(message: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: message.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}
