use crate::api::Backend;
use crate::storage::{repo_jobs, repo_verifiers};
use crate::verifiers::{builtins, runner};
use opscinema_types::{
    AppError, AppErrorCode, AppResult, JobCounters, JobHandle, JobProgress,
    VerifierGetResultRequest, VerifierListRequest, VerifierListResponse, VerifierResultDetail,
    VerifierRunRequest,
};
use serde::Serialize;
use uuid::Uuid;

pub fn verifier_list(
    backend: &Backend,
    req: VerifierListRequest,
) -> AppResult<VerifierListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_verifiers::seed_default_verifiers(&conn).map_err(internal_anyhow)?;
    repo_verifiers::list_verifiers(&conn, req.include_disabled).map_err(internal_anyhow)
}

#[derive(Debug, Serialize)]
struct VerifierRunCompleted {
    run_id: Uuid,
    verifier_id: String,
    status: String,
    result_asset_id: String,
    logs_asset_id: Option<String>,
    evidence_ids: Vec<Uuid>,
}

pub fn verifier_run(backend: &Backend, req: VerifierRunRequest) -> AppResult<JobHandle> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let _ = repo_verifiers::seed_default_verifiers(&conn);
    let job_id = repo_jobs::create_job(&conn, "verifier_run", Some(req.session_id))
        .map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        Some(JobProgress {
            stage: "resolve_spec".to_string(),
            pct: 15,
            counters: JobCounters { done: 0, total: 1 },
        }),
        None,
    );

    let list = match repo_verifiers::list_verifiers(&conn, false).map_err(internal_anyhow) {
        Ok(v) => v,
        Err(err) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                None,
                Some(err.clone()),
            );
            return Err(err);
        }
    };
    let spec = match list
        .verifiers
        .into_iter()
        .find(|v| v.verifier_id == req.verifier_id)
        .ok_or_else(|| not_found("verifier not found"))
    {
        Ok(v) => v,
        Err(err) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                None,
                Some(err.clone()),
            );
            return Err(err);
        }
    };

    let run_outcome = match spec.kind.as_str() {
        "shell" => builtins::shell::run_shell(
            &spec.command_allowlist,
            "echo",
            &["verifier_ok"],
            spec.timeout_secs as u64,
        )
        .map(|out| {
            (
                "SUCCEEDED".to_string(),
                out,
                Some("shell verifier executed allowlisted command".to_string()),
            )
        })
        .map_err(internal_anyhow),
        "file" => {
            let exists = builtins::file::file_exists(std::path::Path::new("/etc/hosts"));
            Ok((
                "SUCCEEDED".to_string(),
                format!("file_exists={exists}"),
                Some("checked file existence for /etc/hosts".to_string()),
            ))
        }
        _ => {
            let err = unsupported("unsupported verifier kind");
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                None,
                Some(err.clone()),
            );
            return Err(err);
        }
    };
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        Some(JobProgress {
            stage: "execute".to_string(),
            pct: 70,
            counters: JobCounters { done: 1, total: 1 },
        }),
        None,
    );

    let (status, output, logs) = match run_outcome {
        Ok(done) => done,
        Err(err) => {
            let fallback = runner::persist_result(
                &conn,
                &backend.assets,
                req.session_id,
                &req.verifier_id,
                "FAILED",
                &format!("verifier execution failed: {}", err.message),
                Some("verifier execution failed"),
            );
            if fallback.is_err() {
                let _ = repo_jobs::update_job_status(
                    &conn,
                    job_id,
                    opscinema_types::JobStatus::Failed,
                    Some(JobProgress {
                        stage: "failed".to_string(),
                        pct: 100,
                        counters: JobCounters { done: 0, total: 1 },
                    }),
                    Some(err.clone()),
                );
                return Err(err);
            }
            let detail = fallback.map_err(internal_anyhow)?;
            let mut conn2 = backend.storage.conn().map_err(db_err)?;
            let payload = VerifierRunCompleted {
                run_id: detail.run_id,
                verifier_id: detail.verifier_id.clone(),
                status: detail.status.clone(),
                result_asset_id: detail.result_asset.asset_id.clone(),
                logs_asset_id: detail.logs_asset.as_ref().map(|a| a.asset_id.clone()),
                evidence_ids: vec![crate::util::ids::deterministic_evidence_id(
                    req.session_id,
                    "VerifierResult",
                    &detail.run_id.to_string(),
                )],
            };
            let _ = crate::storage::event_store::append_event(
                &mut conn2,
                req.session_id,
                "VerifierRunCompleted",
                &payload,
                None,
            )
            .map_err(internal_anyhow)?;
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                Some(JobProgress {
                    stage: "failed".to_string(),
                    pct: 100,
                    counters: JobCounters { done: 0, total: 1 },
                }),
                Some(err.clone()),
            );
            return Err(err);
        }
    };

    let detail = match runner::persist_result(
        &conn,
        &backend.assets,
        req.session_id,
        &req.verifier_id,
        &status,
        &output,
        logs.as_deref(),
    )
    .map_err(internal_anyhow)
    {
        Ok(v) => v,
        Err(err) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Failed,
                Some(JobProgress {
                    stage: "failed".to_string(),
                    pct: 100,
                    counters: JobCounters { done: 0, total: 1 },
                }),
                Some(err.clone()),
            );
            return Err(err);
        }
    };

    let mut conn2 = backend.storage.conn().map_err(db_err)?;
    let payload = VerifierRunCompleted {
        run_id: detail.run_id,
        verifier_id: detail.verifier_id.clone(),
        status: detail.status.clone(),
        result_asset_id: detail.result_asset.asset_id.clone(),
        logs_asset_id: detail.logs_asset.as_ref().map(|a| a.asset_id.clone()),
        evidence_ids: vec![crate::util::ids::deterministic_evidence_id(
            req.session_id,
            "VerifierResult",
            &detail.run_id.to_string(),
        )],
    };
    let _ = crate::storage::event_store::append_event(
        &mut conn2,
        req.session_id,
        "VerifierRunCompleted",
        &payload,
        None,
    )
    .map_err(internal_anyhow)?;

    let terminal = if detail.status.eq_ignore_ascii_case("SUCCEEDED") {
        opscinema_types::JobStatus::Succeeded
    } else {
        opscinema_types::JobStatus::Failed
    };
    let stage = if terminal == opscinema_types::JobStatus::Succeeded {
        "completed".to_string()
    } else {
        "failed".to_string()
    };
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        terminal,
        Some(JobProgress {
            stage,
            pct: 100,
            counters: JobCounters { done: 1, total: 1 },
        }),
        None,
    );

    Ok(JobHandle { job_id })
}

pub fn verifier_get_result(
    backend: &Backend,
    req: VerifierGetResultRequest,
) -> AppResult<VerifierResultDetail> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_verifiers::get_run(&conn, req.run_id)
        .map_err(internal_anyhow)?
        .ok_or_else(|| not_found("verifier result not found"))
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

fn unsupported(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::Unsupported,
        message: msg.to_string(),
        details: None,
        recoverable: true,
        action_hint: None,
    }
}
