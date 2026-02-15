use crate::anchors::debug::debug_anchor;
use crate::anchors::providers::vision::StubVisionAnchorProvider;
use crate::anchors::reacquire::reacquire_anchor;
use crate::api::Backend;
use crate::storage::{repo_jobs, repo_timeline};
use crate::util::canon_json::to_canonical_json;
use opscinema_types::{
    AnchorCandidate, AnchorKind, AnchorsDebugRequest, AnchorsDebugResponse,
    AnchorsListForStepRequest, AnchorsListResponse, AnchorsManualSetRequest,
    AnchorsManualSetResponse, AnchorsReacquireRequest, AppError, AppErrorCode, AppResult,
    JobHandle,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct AnchorCandidatesGeneratedPayload {
    step_id: Uuid,
    anchors: Vec<AnchorCandidate>,
}

#[derive(Debug, Serialize)]
struct AnchorManuallySetPayload {
    anchor_id: Uuid,
    locators: Vec<opscinema_types::EvidenceLocator>,
    manual_note: Option<String>,
}

pub fn anchors_list_for_step(
    backend: &Backend,
    req: AnchorsListForStepRequest,
) -> AppResult<AnchorsListResponse> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let mut anchors = crate::anchors::cache::list_for_step(&conn, req.session_id, req.step_id)
        .map_err(internal_anyhow)?;

    if anchors.is_empty() {
        let anchor_id = Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("{}:{}:anchor:vision", req.session_id, req.step_id).as_bytes(),
        );
        let seed = AnchorCandidate {
            anchor_id,
            step_id: req.step_id,
            kind: AnchorKind::VisionAnchor,
            target_signature: "button:continue".to_string(),
            confidence: 90,
            locators: vec![],
            degraded: false,
        };
        crate::storage::event_store::append_event(
            &mut conn,
            req.session_id,
            "AnchorCandidatesGenerated",
            &AnchorCandidatesGeneratedPayload {
                step_id: req.step_id,
                anchors: vec![seed],
            },
            None,
        )
        .map_err(internal_anyhow)?;
        anchors = crate::anchors::cache::list_for_step(&conn, req.session_id, req.step_id)
            .map_err(internal_anyhow)?;
    }

    Ok(AnchorsListResponse { anchors })
}

pub fn anchors_reacquire(backend: &Backend, req: AnchorsReacquireRequest) -> AppResult<JobHandle> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let job_id = repo_jobs::create_job(&conn, "anchors_reacquire", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        None,
        None,
    );

    let mut anchors = crate::anchors::cache::list_for_step(&conn, req.session_id, req.step_id)
        .map_err(internal_anyhow)?;
    if anchors.is_empty() {
        let err = not_found("anchor not found");
        let _ = repo_jobs::update_job_status(
            &conn,
            job_id,
            opscinema_types::JobStatus::Failed,
            None,
            Some(err.clone()),
        );
        return Err(err);
    }

    let run = (|| -> AppResult<()> {
        let keyframe_png = load_keyframe_png(backend, &conn, req.session_id).ok();
        let provider = StubVisionAnchorProvider;
        for anchor in &mut anchors {
            let result = if let Some(bytes) = keyframe_png.as_deref() {
                reacquire_anchor(&provider, anchor, bytes)
            } else {
                Ok(Err(crate::anchors::types::mark_degraded(
                    anchor,
                    "NO_KEYFRAME",
                )))
            }
            .map_err(internal_anyhow)?;

            match result {
                Ok(mut resolved) => {
                    let provider_json = to_canonical_json(&resolved.resolved_locators)
                        .map_err(|e| internal(&e.to_string()))?;
                    let provider_asset = backend
                        .assets
                        .put(&conn, provider_json.as_bytes(), None)
                        .map_err(internal_anyhow)?;
                    resolved.provider_output_asset_id = Some(provider_asset);
                    crate::storage::event_store::append_event(
                        &mut conn,
                        req.session_id,
                        "AnchorResolved",
                        &resolved,
                        None,
                    )
                    .map_err(internal_anyhow)?;
                }
                Err(degraded) => {
                    crate::storage::event_store::append_event(
                        &mut conn,
                        req.session_id,
                        "AnchorDegraded",
                        &degraded,
                        None,
                    )
                    .map_err(internal_anyhow)?;
                }
            }
        }
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

pub fn anchors_manual_set(
    backend: &Backend,
    req: AnchorsManualSetRequest,
) -> AppResult<AnchorsManualSetResponse> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let anchors =
        crate::anchors::cache::replay_session(&conn, req.session_id).map_err(internal_anyhow)?;
    if !anchors.iter().any(|a| a.anchor_id == req.anchor_id) {
        return Err(not_found("anchor not found"));
    }

    crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "AnchorManuallySet",
        &AnchorManuallySetPayload {
            anchor_id: req.anchor_id,
            locators: req.locators.clone(),
            manual_note: req.note,
        },
        None,
    )
    .map_err(internal_anyhow)?;

    let anchor = crate::anchors::cache::replay_session(&conn, req.session_id)
        .map_err(internal_anyhow)?
        .into_iter()
        .find(|a| a.anchor_id == req.anchor_id)
        .ok_or_else(|| not_found("anchor not found"))?;

    Ok(AnchorsManualSetResponse { anchor })
}

pub fn anchors_debug(
    backend: &Backend,
    req: AnchorsDebugRequest,
) -> AppResult<AnchorsDebugResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let anchor = crate::anchors::cache::list_for_step(&conn, req.session_id, req.step_id)
        .map_err(internal_anyhow)?
        .into_iter()
        .next()
        .ok_or_else(|| not_found("anchor not found"))?;
    Ok(debug_anchor(&anchor))
}

fn load_keyframe_png(
    backend: &Backend,
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<u8>> {
    let frames = repo_timeline::get_keyframes(conn, session_id, 0, i64::MAX)?;
    let frame = frames
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no keyframe"))?;
    let path = backend.assets.path_for(&frame.asset.asset_id);
    Ok(std::fs::read(path)?)
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

fn not_found(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::NotFound,
        message: msg.to_string(),
        details: None,
        recoverable: true,
        action_hint: None,
    }
}
