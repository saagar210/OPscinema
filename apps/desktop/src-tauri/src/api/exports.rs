use crate::api::steps::steps_list;
use crate::api::Backend;
use crate::exports::{tutorial_pack, verify};
use crate::storage::repo_exports;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, ExportResult, ExportVerifyRequest, ExportVerifyResponse,
    ExportsListRequest, ExportsListResponse, StepsListRequest, TutorialExportRequest,
};
use serde::Serialize;

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

pub fn exports_list(backend: &Backend, req: ExportsListRequest) -> AppResult<ExportsListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let exports = repo_exports::list_exports(&conn, req.session_id).map_err(internal_anyhow)?;
    Ok(ExportsListResponse { exports })
}

pub fn export_verify_bundle(
    _backend: &Backend,
    req: ExportVerifyRequest,
) -> AppResult<ExportVerifyResponse> {
    verify::verify_bundle(std::path::Path::new(&req.bundle_path)).map_err(internal_anyhow)
}

pub fn tutorial_export_pack(
    backend: &Backend,
    req: TutorialExportRequest,
) -> AppResult<ExportResult> {
    let steps = steps_list(
        backend,
        StepsListRequest {
            session_id: req.session_id,
        },
    )?
    .steps;
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let degraded_anchor_ids = crate::anchors::cache::replay_session(&conn, req.session_id)
        .map_err(internal_anyhow)?
        .into_iter()
        .filter(|a| a.degraded)
        .map(|a| a.anchor_id.to_string())
        .collect::<Vec<_>>();
    let coverage = crate::evidence::coverage::evaluate(&steps);
    let offline_policy_enforced = backend
        .settings
        .lock()
        .map_err(|_| internal("settings lock poisoned"))?
        .offline_mode;
    let model_pins = crate::api::model_dock::collect_model_pins(backend)?;
    let export = tutorial_pack::export_tutorial_pack(
        req.session_id,
        &steps,
        tutorial_pack::TutorialPackBuildOptions {
            missing_evidence: coverage
                .missing_generated_block_ids
                .iter()
                .map(ToString::to_string)
                .collect(),
            degraded_anchor_ids,
            warnings: vec![],
            model_pins,
            offline_policy_enforced,
        },
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
        "tutorial_pack",
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
            bundle_type: "tutorial_pack".to_string(),
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
