use crate::api::Backend;
use crate::platform::macos::screencapturekit;
use crate::policy::permissions::require_screen_permission;
use crate::storage::event_store::append_event;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, CaptureConfig, CaptureStartRequest, CaptureStatus,
    CaptureStatusRequest, CaptureStopRequest,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use uuid::Uuid;

fn capture_config_store() -> &'static Mutex<CaptureConfig> {
    static STORE: OnceLock<Mutex<CaptureConfig>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(CaptureConfig {
            keyframe_interval_ms: 400,
            include_input: false,
            include_window_meta: false,
        })
    })
}

pub fn capture_get_config() -> AppResult<CaptureConfig> {
    capture_config_store()
        .lock()
        .map(|cfg| cfg.clone())
        .map_err(|_| internal("capture config lock poisoned"))
}

pub fn capture_set_config(cfg: CaptureConfig) -> AppResult<CaptureConfig> {
    let mut guard = capture_config_store()
        .lock()
        .map_err(|_| internal("capture config lock poisoned"))?;
    *guard = cfg.clone();
    Ok(cfg)
}

#[derive(Debug, Serialize)]
struct KeyframeCaptured {
    frame_ms: i64,
    asset_id: String,
    display_id: String,
    pixel_w: u32,
    pixel_h: u32,
    scale_factor: String,
}

#[derive(Debug, Serialize)]
struct ClickCaptured {
    frame_ms: i64,
    button: String,
    pos_norm: PosNorm,
    display_id: String,
}

#[derive(Debug, Serialize)]
struct WindowMetaCaptured {
    frame_ms: i64,
    frontmost_bundle_id: Option<String>,
    frontmost_title: Option<String>,
}

#[derive(Debug, Serialize)]
struct PosNorm {
    x: f32,
    y: f32,
}

pub fn capture_start(backend: &Backend, req: CaptureStartRequest) -> AppResult<CaptureStatus> {
    let perms = crate::api::app::app_get_permissions_status()?;
    require_screen_permission(&perms)?;

    let current_status = backend
        .capture_status
        .lock()
        .map_err(|_| internal("lock poisoned"))?
        .clone();
    if current_status.state == opscinema_types::CaptureState::Capturing
        && current_status.session_id != Some(req.session_id)
    {
        return Err(AppError {
            code: AppErrorCode::Conflict,
            message: "another capture session is already running".to_string(),
            details: current_status
                .session_id
                .map(|id| format!("active_session_id={id}")),
            recoverable: true,
            action_hint: Some("stop current capture before starting a new session".to_string()),
        });
    }

    if let Some(existing) = backend
        .capture_loop
        .lock()
        .map_err(|_| internal("capture loop lock poisoned"))?
        .clone()
    {
        if existing.session_id != req.session_id && !existing.stop.load(Ordering::Relaxed) {
            return Err(AppError {
                code: AppErrorCode::Conflict,
                message: "another capture session is already running".to_string(),
                details: Some(format!("active_session_id={}", existing.session_id)),
                recoverable: true,
                action_hint: Some("stop current capture before starting a new session".to_string()),
            });
        }
    }

    let start_ms = frame_ms_seed(0);
    capture_single_frame(backend, req.session_id, start_ms)?;

    let mut status = backend
        .capture_status
        .lock()
        .map_err(|_| internal("lock poisoned"))?;
    status.state = opscinema_types::CaptureState::Capturing;
    status.session_id = Some(req.session_id);
    status.started_at = Some(chrono::Utc::now());
    let response = status.clone();
    drop(status);

    let interval_ms = capture_get_config()?.keyframe_interval_ms.max(100);
    emit_capture_status_hook(backend, &response);

    let max_frames = capture_loop_max_frames();
    if max_frames != Some(1) {
        spawn_capture_loop(backend, req.session_id, start_ms, interval_ms, max_frames)?;
    }

    Ok(response)
}

pub fn capture_stop(backend: &Backend, req: CaptureStopRequest) -> AppResult<CaptureStatus> {
    let mut should_clear_loop = false;
    if let Some(loop_control) = backend
        .capture_loop
        .lock()
        .map_err(|_| internal("capture loop lock poisoned"))?
        .clone()
    {
        if loop_control.session_id == req.session_id {
            loop_control.stop.store(true, Ordering::Relaxed);
            should_clear_loop = true;
        }
    }
    if should_clear_loop {
        let mut loop_slot = backend
            .capture_loop
            .lock()
            .map_err(|_| internal("capture loop lock poisoned"))?;
        if loop_slot
            .as_ref()
            .map(|loop_ref| loop_ref.session_id == req.session_id)
            .unwrap_or(false)
        {
            *loop_slot = None;
        }
    }

    let mut status = backend
        .capture_status
        .lock()
        .map_err(|_| internal("lock poisoned"))?;
    if status.session_id != Some(req.session_id) {
        return Err(AppError {
            code: AppErrorCode::Conflict,
            message: "capture session mismatch".to_string(),
            details: None,
            recoverable: true,
            action_hint: Some("refresh capture state".to_string()),
        });
    }
    status.state = opscinema_types::CaptureState::Stopped;
    let stopped = status.clone();
    drop(status);
    emit_capture_status_hook(backend, &stopped);
    Ok(stopped)
}

pub fn capture_get_status(
    backend: &Backend,
    _req: CaptureStatusRequest,
) -> AppResult<CaptureStatus> {
    backend
        .capture_status
        .lock()
        .map(|s| s.clone())
        .map_err(|_| internal("lock poisoned"))
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

fn internal(message: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: message.to_string(),
        details: None,
        recoverable: false,
        action_hint: None,
    }
}

fn frame_ms_seed(offset: i64) -> i64 {
    let deterministic = std::env::var("OPSCINEMA_DETERMINISTIC_IDS")
        .map(|v| v == "1")
        .unwrap_or(false);
    if deterministic {
        return offset;
    }
    chrono::Utc::now().timestamp_millis() + offset
}

fn capture_loop_max_frames() -> Option<usize> {
    if let Ok(value) = std::env::var("OPSCINEMA_CAPTURE_BURST_FRAMES") {
        if let Ok(parsed) = value.parse::<usize>() {
            if parsed == 0 {
                return None;
            }
            return Some(parsed.max(1));
        }
    }
    let deterministic = std::env::var("OPSCINEMA_DETERMINISTIC_IDS")
        .map(|v| v == "1")
        .unwrap_or(false);
    if deterministic {
        Some(1)
    } else {
        None
    }
}

fn spawn_capture_loop(
    backend: &Backend,
    session_id: Uuid,
    start_ms: i64,
    interval_ms: u32,
    max_frames: Option<usize>,
) -> AppResult<()> {
    let stop = std::sync::Arc::new(AtomicBool::new(false));
    {
        let mut loop_slot = backend
            .capture_loop
            .lock()
            .map_err(|_| internal("capture loop lock poisoned"))?;
        if let Some(existing) = loop_slot.take() {
            existing.stop.store(true, Ordering::Relaxed);
        }
        *loop_slot = Some(crate::api::CaptureLoopControl {
            session_id,
            stop: stop.clone(),
        });
    }

    let backend_clone = backend.clone();
    std::thread::spawn(move || {
        let mut idx = 1usize;
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if let Some(limit) = max_frames {
                if idx >= limit {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(interval_ms as u64));
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let frame_ms = start_ms + (idx as i64 * i64::from(interval_ms));
            if capture_single_frame(&backend_clone, session_id, frame_ms).is_err() {
                break;
            }
            let capturing = capture_status_snapshot(
                &backend_clone,
                opscinema_types::CaptureState::Capturing,
                Some(session_id),
            );
            emit_capture_status_hook(&backend_clone, &capturing);
            idx += 1;
        }

        if let Ok(mut loop_slot) = backend_clone.capture_loop.lock() {
            if loop_slot
                .as_ref()
                .map(|c| c.session_id == session_id)
                .unwrap_or(false)
            {
                *loop_slot = None;
            }
        }
        if let Ok(status) = backend_clone.capture_status.lock() {
            if status.session_id == Some(session_id)
                && status.state == opscinema_types::CaptureState::Capturing
            {
                emit_capture_status_hook(&backend_clone, &status.clone());
            }
        }
    });

    Ok(())
}

fn capture_status_snapshot(
    backend: &Backend,
    default_state: opscinema_types::CaptureState,
    default_session_id: Option<Uuid>,
) -> CaptureStatus {
    backend
        .capture_status
        .lock()
        .map(|status| status.clone())
        .unwrap_or(CaptureStatus {
            state: default_state,
            session_id: default_session_id,
            started_at: Some(chrono::Utc::now()),
        })
}

fn emit_capture_status_hook(backend: &Backend, status: &CaptureStatus) {
    let hook = backend
        .capture_status_hook
        .lock()
        .ok()
        .and_then(|guard| guard.clone());
    if let Some(callback) = hook {
        callback(status.clone());
    }
}

fn capture_single_frame(backend: &Backend, session_id: Uuid, frame_ms: i64) -> AppResult<()> {
    let provider = screencapturekit::provider();
    let keyframe = provider
        .capture_keyframe(frame_ms)
        .map_err(|e| internal(&format!("capture failed: {e}")))?;

    let mut conn = backend.storage.conn().map_err(db_err)?;
    let asset_id = backend
        .assets
        .put(&conn, &keyframe.png_bytes, None)
        .map_err(|e| internal(&e.to_string()))?;
    let display_id = keyframe.display_id.clone();
    let payload = KeyframeCaptured {
        frame_ms: keyframe.frame_ms,
        asset_id,
        display_id,
        pixel_w: keyframe.pixel_w,
        pixel_h: keyframe.pixel_h,
        scale_factor: keyframe.scale_factor,
    };
    append_event(&mut conn, session_id, "KeyframeCaptured", &payload, None)
        .map_err(|e| internal(&e.to_string()))?;

    let settings = backend
        .settings
        .lock()
        .map_err(|_| internal("settings lock poisoned"))?
        .clone();
    if settings.allow_input_capture {
        let click = crate::capture::input::capture_click(keyframe.frame_ms, &keyframe.display_id);
        append_event(
            &mut conn,
            session_id,
            "ClickCaptured",
            &ClickCaptured {
                frame_ms: click.frame_ms,
                button: click.button,
                pos_norm: PosNorm {
                    x: click.x_norm as f32 / 10_000.0,
                    y: click.y_norm as f32 / 10_000.0,
                },
                display_id: click.display_id,
            },
            None,
        )
        .map_err(|e| internal(&e.to_string()))?;
    }
    if settings.allow_window_metadata {
        let meta = crate::capture::window_meta::capture_window_meta(keyframe.frame_ms);
        append_event(
            &mut conn,
            session_id,
            "WindowMetaCaptured",
            &WindowMetaCaptured {
                frame_ms: meta.frame_ms,
                frontmost_bundle_id: meta.frontmost_bundle_id,
                frontmost_title: meta.frontmost_title,
            },
            None,
        )
        .map_err(|e| internal(&e.to_string()))?;
    }

    Ok(())
}
