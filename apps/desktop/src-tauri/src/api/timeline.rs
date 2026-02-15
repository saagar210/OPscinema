use crate::api::Backend;
use crate::storage::repo_timeline;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, AssetRef, TimelineEventsRequest, TimelineEventsResponse,
    TimelineKeyframesRequest, TimelineKeyframesResponse, TimelineThumbnailRequest,
};

pub fn timeline_get_keyframes(
    backend: &Backend,
    req: TimelineKeyframesRequest,
) -> AppResult<TimelineKeyframesResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let keyframes = repo_timeline::get_keyframes(&conn, req.session_id, req.start_ms, req.end_ms)
        .map_err(internal_anyhow)?;
    Ok(TimelineKeyframesResponse { keyframes })
}

pub fn timeline_get_events(
    backend: &Backend,
    req: TimelineEventsRequest,
) -> AppResult<TimelineEventsResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let events = repo_timeline::get_events(
        &conn,
        req.session_id,
        req.after_seq,
        req.limit.unwrap_or(200),
    )
    .map_err(internal_anyhow)?;
    Ok(TimelineEventsResponse { events })
}

pub fn timeline_get_thumbnail(
    backend: &Backend,
    req: TimelineThumbnailRequest,
) -> AppResult<AssetRef> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let asset_id = repo_timeline::get_thumbnail_asset(&conn, req.session_id, req.frame_event_id)
        .map_err(internal_anyhow)?
        .ok_or_else(|| AppError {
            code: AppErrorCode::NotFound,
            message: "thumbnail not found".to_string(),
            details: None,
            recoverable: true,
            action_hint: None,
        })?;
    Ok(AssetRef { asset_id })
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
