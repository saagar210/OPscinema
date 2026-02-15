use crate::api::Backend;
use crate::storage::repo_sessions;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, SessionCloseRequest, SessionCreateRequest, SessionDetail,
    SessionGetRequest, SessionListRequest, SessionSummary,
};

pub fn session_create(backend: &Backend, req: SessionCreateRequest) -> AppResult<SessionSummary> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_sessions::create_session(&conn, &req.label).map_err(internal_anyhow)
}

pub fn session_list(backend: &Backend, req: SessionListRequest) -> AppResult<Vec<SessionSummary>> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_sessions::list_sessions(&conn, req.limit.unwrap_or(100)).map_err(internal_anyhow)
}

pub fn session_get(backend: &Backend, req: SessionGetRequest) -> AppResult<SessionDetail> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let detail = repo_sessions::get_session(&conn, req.session_id)
        .map_err(internal_anyhow)?
        .ok_or_else(|| not_found("session not found"))?;
    Ok(detail)
}

pub fn session_close(backend: &Backend, req: SessionCloseRequest) -> AppResult<SessionSummary> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let closed = repo_sessions::close_session(&conn, req.session_id).map_err(internal_anyhow)?;
    if !closed {
        return Err(not_found("session not found"));
    }
    session_get(
        backend,
        SessionGetRequest {
            session_id: req.session_id,
        },
    )
    .map(|d| d.summary)
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
    internal(&e.to_string())
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

fn not_found(msg: &str) -> AppError {
    AppError {
        code: AppErrorCode::NotFound,
        message: msg.to_string(),
        details: None,
        recoverable: true,
        action_hint: None,
    }
}
