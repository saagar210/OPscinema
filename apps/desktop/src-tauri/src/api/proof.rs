use crate::api::steps::steps_list;
use crate::api::Backend;
use crate::evidence::coverage;
use crate::exports::{proof_bundle, runbook};
use crate::storage::{repo_exports, repo_sessions};
use opscinema_types::{
    AppError, AppErrorCode, AppResult, ExportResult, ExportWarning, ProofExportRequest,
    ProofGetViewRequest, ProofViewResponse, RunbookCreateRequest, RunbookDetail,
    RunbookExportRequest, RunbookUpdateRequest, Step, StepsListRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunbookCreatedPayload {
    runbook_id: Uuid,
    title: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunbookUpdatedPayload {
    runbook_id: Uuid,
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExportCreatedPayload {
    export_id: uuid::Uuid,
    bundle_type: String,
    output_path: String,
    manifest_asset_id: String,
    bundle_hash: String,
    warnings: Vec<opscinema_types::ExportWarning>,
    created_at: String,
}

pub fn proof_get_view(backend: &Backend, req: ProofGetViewRequest) -> AppResult<ProofViewResponse> {
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let conn = backend.storage.conn().map_err(db_err)?;
    let evidence = crate::evidence::graph::derive_from_event_log(&conn, req.session_id)
        .map_err(internal_anyhow)?;
    Ok(ProofViewResponse {
        steps,
        evidence,
        warnings: vec![],
    })
}

pub fn runbook_create(backend: &Backend, req: RunbookCreateRequest) -> AppResult<RunbookDetail> {
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let runbook_id = Uuid::new_v4();
    let detail = RunbookDetail {
        runbook_id,
        title: req.title,
        steps,
    };
    let mut conn = backend.storage.conn().map_err(db_err)?;
    crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "RunbookCreated",
        &RunbookCreatedPayload {
            runbook_id,
            title: detail.title.clone(),
            steps: detail.steps.clone(),
        },
        None,
    )
    .map_err(internal_anyhow)?;
    Ok(detail)
}

pub fn runbook_update(backend: &Backend, req: RunbookUpdateRequest) -> AppResult<RunbookDetail> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let (session_id, existing) = find_runbook(&conn, req.runbook_id).map_err(internal_anyhow)?;
    crate::storage::event_store::append_event(
        &mut conn,
        session_id,
        "RunbookUpdated",
        &RunbookUpdatedPayload {
            runbook_id: req.runbook_id,
            title: req.title,
        },
        None,
    )
    .map_err(internal_anyhow)?;
    let updated = replay_runbooks_for_session(&conn, session_id)
        .map_err(internal_anyhow)?
        .remove(&req.runbook_id)
        .unwrap_or(existing);
    Ok(updated)
}

pub fn runbook_export(backend: &Backend, req: RunbookExportRequest) -> AppResult<ExportResult> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let (session_id, detail) = find_runbook(&conn, req.runbook_id).map_err(internal_anyhow)?;
    let coverage = coverage::evaluate(&detail.steps);
    let warnings = collect_verifier_warnings(&conn, session_id).map_err(internal_anyhow)?;
    let offline_policy_enforced = backend
        .settings
        .lock()
        .map_err(|_| internal("settings lock poisoned"))?
        .offline_mode;
    let model_pins = crate::api::model_dock::collect_model_pins(backend)?;
    let export = runbook::export_runbook(
        session_id,
        &detail,
        &warnings,
        coverage
            .missing_generated_block_ids
            .iter()
            .map(ToString::to_string)
            .collect(),
        model_pins,
        offline_policy_enforced,
        std::path::Path::new(&req.output_dir),
    )
    .map_err(internal_anyhow)?;

    let manifest_path = std::path::Path::new(&export.output_path).join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path).map_err(|e| internal(&e.to_string()))?;
    let manifest_asset_id = backend
        .assets
        .put(&conn, &manifest_bytes, None)
        .map_err(internal_anyhow)?;
    let persisted = repo_exports::insert_export(
        &conn,
        session_id,
        "runbook",
        &export.output_path,
        &manifest_asset_id,
        &export.bundle_hash,
        &export.warnings,
    )
    .map_err(internal_anyhow)?;
    let _ = crate::storage::event_store::append_event(
        &mut conn,
        session_id,
        "ExportCreated",
        &ExportCreatedPayload {
            export_id: persisted.export_id,
            bundle_type: "runbook".to_string(),
            output_path: persisted.output_path.clone(),
            manifest_asset_id,
            bundle_hash: persisted.bundle_hash.clone(),
            warnings: persisted.warnings.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        None,
    )
    .map_err(internal_anyhow)?;
    Ok(persisted)
}

pub fn proof_export_bundle(backend: &Backend, req: ProofExportRequest) -> AppResult<ExportResult> {
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let coverage = coverage::evaluate(&steps);
    let warnings = collect_verifier_warnings(&conn, req.session_id).map_err(internal_anyhow)?;
    let offline_policy_enforced = backend
        .settings
        .lock()
        .map_err(|_| internal("settings lock poisoned"))?
        .offline_mode;
    let model_pins = crate::api::model_dock::collect_model_pins(backend)?;
    let export = proof_bundle::export_proof_bundle(
        req.session_id,
        &steps,
        &warnings,
        coverage
            .missing_generated_block_ids
            .iter()
            .map(ToString::to_string)
            .collect(),
        model_pins,
        offline_policy_enforced,
        std::path::Path::new(&req.output_dir),
    )
    .map_err(internal_anyhow)?;
    let manifest_path = std::path::Path::new(&export.output_path).join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path).map_err(|e| internal(&e.to_string()))?;
    let manifest_asset_id = backend
        .assets
        .put(&conn, &manifest_bytes, None)
        .map_err(internal_anyhow)?;
    let persisted = repo_exports::insert_export(
        &conn,
        req.session_id,
        "proof_bundle",
        &export.output_path,
        &manifest_asset_id,
        &export.bundle_hash,
        &export.warnings,
    )
    .map_err(internal_anyhow)?;
    let _ = crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "ExportCreated",
        &ExportCreatedPayload {
            export_id: persisted.export_id,
            bundle_type: "proof_bundle".to_string(),
            output_path: persisted.output_path.clone(),
            manifest_asset_id,
            bundle_hash: persisted.bundle_hash.clone(),
            warnings: persisted.warnings.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        },
        None,
    )
    .map_err(internal_anyhow)?;
    Ok(persisted)
}

#[derive(Debug, Deserialize)]
struct VerifierRunCompletedPayload {
    verifier_id: String,
    status: String,
}

fn collect_verifier_warnings(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<Vec<ExportWarning>> {
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
        warnings.push(ExportWarning {
            code: "VERIFIER_WARN".to_string(),
            message: format!(
                "Verifier {} completed with status {}",
                payload.verifier_id, payload.status
            ),
        });
    }
    Ok(warnings)
}

fn replay_runbooks_for_session(
    conn: &crate::storage::DbConn,
    session_id: Uuid,
) -> anyhow::Result<BTreeMap<Uuid, RunbookDetail>> {
    let events = crate::storage::event_store::query_events(conn, session_id, None, 100_000)?;
    let mut runbooks = BTreeMap::<Uuid, RunbookDetail>::new();

    for event in events {
        match event.event_type.as_str() {
            "RunbookCreated" => {
                let payload: RunbookCreatedPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                runbooks.insert(
                    payload.runbook_id,
                    RunbookDetail {
                        runbook_id: payload.runbook_id,
                        title: payload.title,
                        steps: payload.steps,
                    },
                );
            }
            "RunbookUpdated" => {
                let payload: RunbookUpdatedPayload =
                    serde_json::from_str(&event.payload_canon_json)?;
                if let Some(detail) = runbooks.get_mut(&payload.runbook_id) {
                    if let Some(title) = payload.title {
                        detail.title = title;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(runbooks)
}

fn find_runbook(
    conn: &crate::storage::DbConn,
    runbook_id: Uuid,
) -> anyhow::Result<(Uuid, RunbookDetail)> {
    let sessions = repo_sessions::list_sessions(conn, 10_000)?;
    for session in sessions {
        let runbooks = replay_runbooks_for_session(conn, session.session_id)?;
        if let Some(detail) = runbooks.get(&runbook_id) {
            return Ok((session.session_id, detail.clone()));
        }
    }
    anyhow::bail!("runbook not found");
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
    let message = e.to_string();
    if message.contains("runbook not found") {
        return AppError {
            code: AppErrorCode::NotFound,
            message: "runbook not found".to_string(),
            details: None,
            recoverable: true,
            action_hint: None,
        };
    }
    if message.contains("ExportGateFailed") {
        return AppError {
            code: AppErrorCode::ExportGateFailed,
            message: "proof/runbook export blocked by policy gate".to_string(),
            details: Some(message),
            recoverable: true,
            action_hint: Some("Attach evidence refs and retry export".to_string()),
        };
    }
    internal(&message)
}
