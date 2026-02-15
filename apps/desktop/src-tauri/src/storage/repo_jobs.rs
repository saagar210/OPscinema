use crate::util::time::now_utc_iso;
use opscinema_types::{AppError, JobDetail, JobProgress, JobStatus};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

pub fn create_job(
    conn: &rusqlite::Connection,
    job_type: &str,
    session_id: Option<Uuid>,
) -> anyhow::Result<Uuid> {
    let job_id = Uuid::new_v4();
    conn.execute(
        "INSERT INTO jobs(job_id, job_type, session_id, status, created_at)
         VALUES (?1, ?2, ?3, 'QUEUED', ?4)",
        params![
            job_id.to_string(),
            job_type,
            session_id.map(|s| s.to_string()),
            now_utc_iso()
        ],
    )?;
    Ok(job_id)
}

pub fn update_job_status(
    conn: &rusqlite::Connection,
    job_id: Uuid,
    status: JobStatus,
    progress: Option<JobProgress>,
    error: Option<AppError>,
) -> anyhow::Result<()> {
    let now = now_utc_iso();
    let status_db = status_to_db(status);
    conn.execute(
        "UPDATE jobs
         SET status=?2,
             progress_json=?3,
             error_json=?4,
             started_at=CASE WHEN ?2='RUNNING' AND started_at IS NULL THEN ?5 ELSE started_at END,
             ended_at=CASE WHEN ?2 IN ('SUCCEEDED','FAILED','CANCELLED') THEN ?6 ELSE ended_at END
         WHERE job_id=?1",
        params![
            job_id.to_string(),
            status_db,
            progress.map(|p| serde_json::to_string(&p)).transpose()?,
            error.map(|e| serde_json::to_string(&e)).transpose()?,
            now,
            now_utc_iso()
        ],
    )?;
    Ok(())
}

pub fn list_jobs(
    conn: &rusqlite::Connection,
    session_id: Option<Uuid>,
    status: Option<JobStatus>,
) -> anyhow::Result<Vec<JobDetail>> {
    let status_db = status.map(status_to_db);
    let mut stmt = conn.prepare(
        "SELECT job_id, job_type, session_id, status, created_at, started_at, ended_at, progress_json, error_json
         FROM jobs
         WHERE (?1 IS NULL OR session_id=?1)
           AND (?2 IS NULL OR status=?2)
         ORDER BY created_at DESC",
    )?;
    let rows = stmt
        .query_map(
            params![session_id.map(|s| s.to_string()), status_db],
            parse_job_detail_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn get_job(conn: &rusqlite::Connection, job_id: Uuid) -> anyhow::Result<Option<JobDetail>> {
    let row = conn
        .query_row(
            "SELECT job_id, job_type, session_id, status, created_at, started_at, ended_at, progress_json, error_json
             FROM jobs WHERE job_id=?1",
            params![job_id.to_string()],
            parse_job_detail_row,
        )
        .optional()?;
    Ok(row)
}

pub fn cancel_job(conn: &rusqlite::Connection, job_id: Uuid) -> anyhow::Result<bool> {
    let now = now_utc_iso();
    let updated = conn.execute(
        "UPDATE jobs
         SET status='CANCELLED',
             ended_at=COALESCE(ended_at, ?2)
         WHERE job_id=?1
           AND status IN ('QUEUED', 'RUNNING', 'CANCELLED')",
        params![job_id.to_string(), now],
    )?;
    if updated > 0 {
        return Ok(true);
    }

    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM jobs WHERE job_id=?1",
            params![job_id.to_string()],
            |r| r.get(0),
        )
        .optional()?;
    Ok(exists.is_some())
}

fn parse_job_detail_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<JobDetail> {
    let status_raw: String = r.get(3)?;
    let status = status_from_db(&status_raw);
    Ok(JobDetail {
        job_id: Uuid::parse_str(&r.get::<_, String>(0)?).unwrap_or_else(|_| Uuid::nil()),
        job_type: r.get(1)?,
        session_id: r
            .get::<_, Option<String>>(2)?
            .and_then(|s| Uuid::parse_str(&s).ok()),
        status,
        created_at: chrono::DateTime::parse_from_rfc3339(&r.get::<_, String>(4)?)
            .map(|v| v.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now()),
        started_at: r
            .get::<_, Option<String>>(5)?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        ended_at: r
            .get::<_, Option<String>>(6)?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        progress: r
            .get::<_, Option<String>>(7)?
            .and_then(|s| serde_json::from_str(&s).ok()),
        error: r
            .get::<_, Option<String>>(8)?
            .and_then(|s| serde_json::from_str(&s).ok()),
    })
}

fn status_from_db(raw: &str) -> JobStatus {
    match raw {
        "QUEUED" => JobStatus::Queued,
        "RUNNING" => JobStatus::Running,
        "SUCCEEDED" => JobStatus::Succeeded,
        "FAILED" => JobStatus::Failed,
        _ => JobStatus::Cancelled,
    }
}

fn status_to_db(status: JobStatus) -> &'static str {
    match status {
        JobStatus::Queued => "QUEUED",
        JobStatus::Running => "RUNNING",
        JobStatus::Succeeded => "SUCCEEDED",
        JobStatus::Failed => "FAILED",
        JobStatus::Cancelled => "CANCELLED",
    }
}
