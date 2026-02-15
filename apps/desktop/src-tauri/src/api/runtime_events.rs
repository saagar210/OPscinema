use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use opscinema_types::{
    AppError, AppErrorCode, AppResult, CaptureStatus, CaptureStatusEvent, EventStreamEnvelope,
    JobCounters, JobId, JobProgressEvent, JobStatus, JobStatusEvent,
};
use tauri::{AppHandle, Emitter, Wry};

#[derive(Debug, Clone, Default)]
pub struct RuntimeEventBus {
    app: Option<AppHandle<Wry>>,
    job_progress_seq: Arc<AtomicU64>,
    job_status_seq: Arc<AtomicU64>,
    capture_status_seq: Arc<AtomicU64>,
}

impl RuntimeEventBus {
    pub fn with_app(app: AppHandle<Wry>) -> Self {
        Self {
            app: Some(app),
            ..Self::default()
        }
    }

    pub fn emit_job_queued(&self, job_id: JobId) -> AppResult<()> {
        self.emit_job_status(JobStatusEvent {
            job_id,
            status: JobStatus::Queued,
        })?;
        self.emit_job_progress_stage(job_id, "queued", 0, 0, 0)
    }

    pub fn emit_job_running(&self, job_id: JobId) -> AppResult<()> {
        self.emit_job_status(JobStatusEvent {
            job_id,
            status: JobStatus::Running,
        })?;
        self.emit_job_progress_stage(job_id, "running", 50, 0, 0)
    }

    pub fn emit_job_succeeded(&self, job_id: JobId) -> AppResult<()> {
        self.emit_job_progress_stage(job_id, "completed", 100, 1, 1)?;
        self.emit_job_status(JobStatusEvent {
            job_id,
            status: JobStatus::Succeeded,
        })
    }

    pub fn emit_job_failed(&self, job_id: JobId) -> AppResult<()> {
        self.emit_job_progress_stage(job_id, "failed", 100, 0, 1)?;
        self.emit_job_status(JobStatusEvent {
            job_id,
            status: JobStatus::Failed,
        })
    }

    pub fn emit_job_progress_stage(
        &self,
        job_id: JobId,
        stage: &str,
        pct: u8,
        done: u64,
        total: u64,
    ) -> AppResult<()> {
        self.emit_job_progress(JobProgressEvent {
            job_id,
            stage: stage.to_string(),
            pct,
            counters: JobCounters { done, total },
        })
    }

    pub fn emit_job_cancelled(&self, job_id: JobId) -> AppResult<()> {
        self.emit_job_status(JobStatusEvent {
            job_id,
            status: JobStatus::Cancelled,
        })
    }

    pub fn emit_capture_status(&self, status: &CaptureStatus) -> AppResult<()> {
        self.emit_capture(CaptureStatusEvent {
            state: status.state.clone(),
            session_id: status.session_id,
        })
    }

    fn emit_job_progress(&self, payload: JobProgressEvent) -> AppResult<()> {
        let app = self.app_handle()?;
        let envelope = EventStreamEnvelope {
            stream_seq: self.job_progress_seq.fetch_add(1, Ordering::Relaxed) + 1,
            sent_at: Utc::now(),
            payload,
        };
        app.emit("job_progress", envelope)
            .map_err(|e| internal(&e.to_string()))
    }

    fn emit_job_status(&self, payload: JobStatusEvent) -> AppResult<()> {
        let app = self.app_handle()?;
        let envelope = EventStreamEnvelope {
            stream_seq: self.job_status_seq.fetch_add(1, Ordering::Relaxed) + 1,
            sent_at: Utc::now(),
            payload,
        };
        app.emit("job_status", envelope)
            .map_err(|e| internal(&e.to_string()))
    }

    fn emit_capture(&self, payload: CaptureStatusEvent) -> AppResult<()> {
        let app = self.app_handle()?;
        let envelope = EventStreamEnvelope {
            stream_seq: self.capture_status_seq.fetch_add(1, Ordering::Relaxed) + 1,
            sent_at: Utc::now(),
            payload,
        };
        app.emit("capture_status", envelope)
            .map_err(|e| internal(&e.to_string()))
    }

    fn app_handle(&self) -> AppResult<&AppHandle<Wry>> {
        self.app
            .as_ref()
            .ok_or_else(|| internal("runtime handle unavailable"))
    }

    #[cfg(test)]
    pub(crate) fn next_job_status_seq_for_test(&self) -> u64 {
        self.job_status_seq.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[cfg(test)]
    pub(crate) fn next_job_progress_seq_for_test(&self) -> u64 {
        self.job_progress_seq.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[cfg(test)]
    pub(crate) fn next_capture_status_seq_for_test(&self) -> u64 {
        self.capture_status_seq.fetch_add(1, Ordering::Relaxed) + 1
    }
}

#[allow(dead_code)]
pub(crate) fn lifecycle_status_order(current: JobStatus) -> Vec<JobStatus> {
    match current {
        JobStatus::Queued => vec![JobStatus::Queued],
        JobStatus::Running => vec![JobStatus::Queued, JobStatus::Running],
        JobStatus::Succeeded => vec![JobStatus::Queued, JobStatus::Running, JobStatus::Succeeded],
        JobStatus::Failed => vec![JobStatus::Queued, JobStatus::Running, JobStatus::Failed],
        JobStatus::Cancelled => vec![JobStatus::Queued, JobStatus::Cancelled],
    }
}

fn internal(message: &str) -> AppError {
    AppError {
        code: AppErrorCode::Internal,
        message: "failed to emit runtime event".to_string(),
        details: Some(message.to_string()),
        recoverable: true,
        action_hint: Some("retry action".to_string()),
    }
}
