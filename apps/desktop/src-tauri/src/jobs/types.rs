use opscinema_types::{JobDetail, JobId};

#[derive(Debug, Clone)]
pub struct JobContext {
    pub job: JobDetail,
    pub cancelled: bool,
}

impl JobContext {
    pub fn should_cancel(&self) -> bool {
        self.cancelled
    }

    pub fn id(&self) -> JobId {
        self.job.job_id
    }
}
