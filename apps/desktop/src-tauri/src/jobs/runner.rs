use crate::jobs::cancel::CancellationSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct JobRunner {
    cancels: Arc<Mutex<CancellationSet>>,
}

impl JobRunner {
    pub fn cancel(&self, job_id: Uuid) {
        if let Ok(mut c) = self.cancels.lock() {
            c.cancel(job_id);
        }
    }

    pub fn is_cancelled(&self, job_id: Uuid) -> bool {
        self.cancels
            .lock()
            .map(|c| c.is_cancelled(job_id))
            .unwrap_or(true)
    }
}
