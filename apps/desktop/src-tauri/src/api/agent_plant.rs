use crate::agent_plant::{dag::PipelineDag, diagnostics, transforms};
use crate::api::Backend;
use crate::storage::{repo_jobs, repo_sessions};
use crate::util::canon_json::to_canonical_json;
use opscinema_types::{
    AgentPipelineReportRequest, AgentPipelineReportResponse, AgentPipelineRunRequest,
    AgentPipelinesListResponse, AppError, AppErrorCode, AppResult, JobHandle, JobStatus,
    StepEditOp,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct StepEditAppliedPayload {
    base_seq: i64,
    op: opscinema_types::StepEditOp,
    applied_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentPipelineRunCompletedPayload {
    run_id: Uuid,
    pipeline_id: String,
    diagnostics_asset_id: String,
}

pub fn agent_pipelines_list() -> AppResult<AgentPipelinesListResponse> {
    Ok(AgentPipelinesListResponse {
        pipelines: vec!["normalize_titles".to_string()],
    })
}

pub fn agent_pipeline_run(backend: &Backend, req: AgentPipelineRunRequest) -> AppResult<JobHandle> {
    if req.pipeline_id != "normalize_titles" {
        return Err(AppError {
            code: AppErrorCode::Unsupported,
            message: "pipeline is not supported".to_string(),
            details: Some(req.pipeline_id),
            recoverable: true,
            action_hint: Some("Use normalize_titles".to_string()),
        });
    }

    let mut conn = backend.storage.conn().map_err(db_err)?;
    let run_id = repo_jobs::create_job(&conn, "agent_pipeline_run", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(&conn, run_id, JobStatus::Running, None, None);

    let existing_steps = crate::steps::replay::replay_session_steps(&conn, req.session_id)
        .map_err(internal_anyhow)?;
    let mut transformed = existing_steps.clone();

    let dag = PipelineDag {
        nodes: BTreeSet::from(["normalize_titles".to_string()]),
        edges: BTreeMap::new(),
    };
    let mut visited = Vec::new();
    for node in dag.topological() {
        if node == "normalize_titles" {
            transforms::normalize_titles::apply(&mut transformed);
            visited.push(node);
        }
    }

    let mut base_seq =
        repo_sessions::get_head_seq(&conn, req.session_id).map_err(internal_anyhow)?;
    for (before, after) in existing_steps.iter().zip(transformed.iter()) {
        if before.title != after.title {
            let op = StepEditOp::UpdateTitle {
                step_id: before.step_id,
                title: after.title.clone(),
            };
            let (_event_id, next_seq, _hash) = crate::storage::event_store::append_event(
                &mut conn,
                req.session_id,
                "StepEditApplied",
                &StepEditAppliedPayload {
                    base_seq,
                    op,
                    applied_at: chrono::Utc::now().to_rfc3339(),
                },
                None,
            )
            .map_err(internal_anyhow)?;
            base_seq = next_seq;
        }
    }

    let diagnostics = diagnostics::summarize(&run_id.to_string(), &visited);
    let diagnostics_json = to_canonical_json(&diagnostics).map_err(|e| internal(&e.to_string()))?;
    let diagnostics_asset_id = backend
        .assets
        .put(&conn, diagnostics_json.as_bytes(), None)
        .map_err(internal_anyhow)?;
    crate::storage::event_store::append_event(
        &mut conn,
        req.session_id,
        "AgentPipelineRunCompleted",
        &AgentPipelineRunCompletedPayload {
            run_id,
            pipeline_id: "normalize_titles".to_string(),
            diagnostics_asset_id,
        },
        None,
    )
    .map_err(internal_anyhow)?;

    let _ = repo_jobs::update_job_status(&conn, run_id, JobStatus::Succeeded, None, None);
    Ok(JobHandle { job_id: run_id })
}

pub fn agent_pipeline_report(
    backend: &Backend,
    req: AgentPipelineReportRequest,
) -> AppResult<AgentPipelineReportResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let sessions =
        crate::storage::repo_sessions::list_sessions(&conn, 10_000).map_err(internal_anyhow)?;
    for session in sessions {
        let events =
            crate::storage::event_store::query_events(&conn, session.session_id, None, 100_000)
                .map_err(internal_anyhow)?;
        for event in events {
            if event.event_type != "AgentPipelineRunCompleted" {
                continue;
            }
            let payload: AgentPipelineRunCompletedPayload =
                serde_json::from_str(&event.payload_canon_json)
                    .map_err(|e| internal(&e.to_string()))?;
            if payload.run_id != req.run_id {
                continue;
            }
            let path = backend.assets.path_for(&payload.diagnostics_asset_id);
            let raw = std::fs::read_to_string(path).map_err(|e| internal(&e.to_string()))?;
            let diagnostics: Vec<String> =
                serde_json::from_str(&raw).map_err(|e| internal(&e.to_string()))?;
            return Ok(AgentPipelineReportResponse {
                run_id: req.run_id,
                diagnostics,
            });
        }
    }

    Err(AppError {
        code: AppErrorCode::NotFound,
        message: "agent pipeline run not found".to_string(),
        details: None,
        recoverable: true,
        action_hint: None,
    })
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
