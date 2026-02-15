use crate::api::Backend;
use crate::ocr::pipeline::persist_ocr_for_frame;
use crate::platform::macos::screencapturekit::capture;
use crate::platform::macos::vision_ocr;
use crate::storage::{repo_jobs, repo_ocr, repo_timeline};
use opscinema_types::{
    AppError, AppErrorCode, AppResult, JobCounters, JobHandle, JobProgress,
    OcrBlocksForFrameRequest, OcrBlocksForFrameResponse, OcrScheduleRequest, OcrSearchRequest,
    OcrSearchResponse, OcrStatus, OcrStatusRequest,
};
use uuid::Uuid;

pub fn ocr_schedule(backend: &Backend, req: OcrScheduleRequest) -> AppResult<JobHandle> {
    let mut conn = backend.storage.conn().map_err(db_err)?;
    let job_id =
        repo_jobs::create_job(&conn, "ocr", Some(req.session_id)).map_err(internal_anyhow)?;
    let _ = repo_jobs::update_job_status(
        &conn,
        job_id,
        opscinema_types::JobStatus::Running,
        Some(JobProgress {
            stage: "scan_keyframes".to_string(),
            pct: 10,
            counters: JobCounters { done: 0, total: 1 },
        }),
        None,
    );

    if backend.jobs.is_cancelled(job_id) {
        let cancel = cancelled();
        let _ = repo_jobs::update_job_status(
            &conn,
            job_id,
            opscinema_types::JobStatus::Cancelled,
            None,
            Some(cancel.clone()),
        );
        return Err(cancel);
    }

    let run = (|| -> AppResult<()> {
        let start_ms = req.start_ms.unwrap_or(0);
        let end_ms = req.end_ms.unwrap_or(i64::MAX);
        let keyframes = repo_timeline::get_keyframes(&conn, req.session_id, start_ms, end_ms)
            .map_err(internal_anyhow)?;
        let _ = repo_jobs::update_job_status(
            &conn,
            job_id,
            opscinema_types::JobStatus::Running,
            Some(JobProgress {
                stage: "prepare_frame".to_string(),
                pct: 30,
                counters: JobCounters {
                    done: 0,
                    total: keyframes.len().max(1) as u64,
                },
            }),
            None,
        );

        let (frame_event_id, frame) = if let Some(existing) = keyframes.into_iter().next() {
            let bytes = std::fs::read(backend.assets.path_for(&existing.asset.asset_id))
                .map_err(|e| internal_anyhow(anyhow::anyhow!(e)))?;
            (
                existing.frame_event_id,
                crate::capture::screen::ScreenKeyframe {
                    frame_ms: existing.frame_ms,
                    display_id: std::env::var("OPSCINEMA_CAPTURE_DISPLAY_ID")
                        .unwrap_or_else(|_| "display.main".to_string()),
                    pixel_w: std::env::var("OPSCINEMA_CAPTURE_PIXEL_W")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(1920),
                    pixel_h: std::env::var("OPSCINEMA_CAPTURE_PIXEL_H")
                        .ok()
                        .and_then(|v| v.parse::<u32>().ok())
                        .unwrap_or(1080),
                    scale_factor: std::env::var("OPSCINEMA_CAPTURE_SCALE")
                        .unwrap_or_else(|_| "2.0".to_string()),
                    png_bytes: bytes,
                },
            )
        } else {
            let captured = capture(start_ms).map_err(internal_anyhow)?;
            (Uuid::new_v4(), captured)
        };

        if backend.jobs.is_cancelled(job_id) {
            return Err(cancelled());
        }

        let provider = vision_ocr::provider();
        let _ = repo_jobs::update_job_status(
            &conn,
            job_id,
            opscinema_types::JobStatus::Running,
            Some(JobProgress {
                stage: "ocr_inference".to_string(),
                pct: 70,
                counters: JobCounters { done: 1, total: 1 },
            }),
            None,
        );
        let _ = persist_ocr_for_frame(
            &mut conn,
            &backend.assets,
            provider.as_ref(),
            req.session_id,
            frame_event_id,
            &frame,
        )
        .map_err(provider_or_internal)?;

        if backend.jobs.is_cancelled(job_id) {
            return Err(cancelled());
        }
        Ok(())
    })();
    match run {
        Ok(()) => {
            let _ = repo_jobs::update_job_status(
                &conn,
                job_id,
                opscinema_types::JobStatus::Succeeded,
                Some(JobProgress {
                    stage: "completed".to_string(),
                    pct: 100,
                    counters: JobCounters { done: 1, total: 1 },
                }),
                None,
            );
            Ok(JobHandle { job_id })
        }
        Err(err) => {
            let terminal = if err.code == AppErrorCode::JobCancelled {
                opscinema_types::JobStatus::Cancelled
            } else {
                opscinema_types::JobStatus::Failed
            };
            let stage = if terminal == opscinema_types::JobStatus::Cancelled {
                "cancelled".to_string()
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
                    counters: JobCounters { done: 0, total: 1 },
                }),
                Some(err.clone()),
            );
            Err(err)
        }
    }
}

pub fn ocr_get_status(_backend: &Backend, _req: OcrStatusRequest) -> AppResult<OcrStatus> {
    let conn = _backend.storage.conn().map_err(db_err)?;
    repo_ocr::status(&conn, _req.session_id).map_err(internal_anyhow)
}

pub fn ocr_search(backend: &Backend, req: OcrSearchRequest) -> AppResult<OcrSearchResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    repo_ocr::search(&conn, req.session_id, &req.query).map_err(internal_anyhow)
}

pub fn ocr_get_blocks_for_frame(
    backend: &Backend,
    req: OcrBlocksForFrameRequest,
) -> AppResult<OcrBlocksForFrameResponse> {
    let conn = backend.storage.conn().map_err(db_err)?;
    let blocks = repo_ocr::list_blocks_for_frame(&conn, req.session_id, req.frame_event_id)
        .map_err(internal_anyhow)?;
    Ok(OcrBlocksForFrameResponse { blocks })
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

fn provider_or_internal(e: anyhow::Error) -> AppError {
    let message = e.to_string();
    if message.to_lowercase().contains("provider schema invalid") {
        return AppError {
            code: AppErrorCode::ProviderSchemaInvalid,
            message: "OCR provider output failed schema validation".to_string(),
            details: Some(message),
            recoverable: true,
            action_hint: Some("Retry OCR or switch provider mode".to_string()),
        };
    }
    internal_anyhow(e)
}

fn cancelled() -> AppError {
    AppError {
        code: AppErrorCode::JobCancelled,
        message: "OCR job cancelled".to_string(),
        details: None,
        recoverable: true,
        action_hint: Some("Retry OCR job".to_string()),
    }
}
