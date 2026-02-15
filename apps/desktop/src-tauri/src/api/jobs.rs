use crate::api::Backend;
use crate::storage::repo_jobs;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, JobDetail, JobsCancelRequest, JobsCancelResponse,
    JobsGetRequest, JobsListRequest, JobsListResponse,
};

pub fn jobs_list(backend: &Backend, req: JobsListRequest) -> AppResult<JobsListResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let jobs = repo_jobs::list_jobs(&conn, req.session_id, req.status).map_err(internal_anyhow)?;
    Ok(JobsListResponse { jobs })
}

pub fn jobs_get(backend: &Backend, req: JobsGetRequest) -> AppResult<JobDetail> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_jobs::get_job(&conn, req.job_id)
        .map_err(internal_anyhow)?
        .ok_or_else(|| AppError {
            code: AppErrorCode::NotFound,
            message: "job not found".to_string(),
            details: None,
            recoverable: true,
            action_hint: None,
        })
}

pub fn jobs_cancel(backend: &Backend, req: JobsCancelRequest) -> AppResult<JobsCancelResponse> {
    backend.jobs.cancel(req.job_id);
    let conn = backend.storage.conn().map_err(db_err)?;
    let accepted = repo_jobs::cancel_job(&conn, req.job_id).map_err(internal_anyhow)?;
    Ok(JobsCancelResponse { accepted })
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
