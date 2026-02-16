use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{PipelineError, Result};
use crate::models::{Job, JobStatus, JobType, Priority};

pub struct CreateJobRequest {
    pub job_type: JobType,
    pub law_id: String,
    pub priority: Priority,
    pub payload: Option<serde_json::Value>,
    pub max_attempts: i32,
}

impl CreateJobRequest {
    pub fn new(job_type: JobType, law_id: impl Into<String>) -> Self {
        Self {
            job_type,
            law_id: law_id.into(),
            priority: Priority::default(),
            payload: None,
            max_attempts: 3,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: i32) -> Self {
        self.max_attempts = max_attempts;
        self
    }
}

/// Create a new job in the queue.
pub async fn create_job(pool: &PgPool, req: CreateJobRequest) -> Result<Job> {
    let job = sqlx::query_as::<_, Job>(
        r#"
        INSERT INTO jobs (job_type, law_id, priority, payload, max_attempts)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(req.job_type)
    .bind(&req.law_id)
    .bind(req.priority.value())
    .bind(&req.payload)
    .bind(req.max_attempts)
    .fetch_one(pool)
    .await?;

    Ok(job)
}

/// Claim the highest-priority pending job using FOR UPDATE SKIP LOCKED.
/// Returns None if no jobs are available.
pub async fn claim_job(pool: &PgPool, job_type: Option<JobType>) -> Result<Option<Job>> {
    let job = match job_type {
        Some(jt) => {
            sqlx::query_as::<_, Job>(
                r#"
                UPDATE jobs
                SET status = 'processing', started_at = now(), attempts = attempts + 1
                WHERE id = (
                    SELECT id FROM jobs
                    WHERE status = 'pending' AND job_type = $1
                    ORDER BY priority DESC, created_at ASC
                    LIMIT 1
                    FOR UPDATE SKIP LOCKED
                )
                RETURNING *
                "#,
            )
            .bind(jt)
            .fetch_optional(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Job>(
                r#"
                UPDATE jobs
                SET status = 'processing', started_at = now(), attempts = attempts + 1
                WHERE id = (
                    SELECT id FROM jobs
                    WHERE status = 'pending'
                    ORDER BY priority DESC, created_at ASC
                    LIMIT 1
                    FOR UPDATE SKIP LOCKED
                )
                RETURNING *
                "#,
            )
            .fetch_optional(pool)
            .await?
        }
    };

    Ok(job)
}

/// Mark a job as completed with an optional result payload.
pub async fn complete_job(
    pool: &PgPool,
    job_id: Uuid,
    result: Option<serde_json::Value>,
) -> Result<Job> {
    let job = sqlx::query_as::<_, Job>(
        r#"
        UPDATE jobs
        SET status = 'completed', completed_at = now(), result = $2
        WHERE id = $1 AND status = 'processing'
        RETURNING *
        "#,
    )
    .bind(job_id)
    .bind(&result)
    .fetch_optional(pool)
    .await?
    .ok_or(PipelineError::JobNotFound(job_id))?;

    Ok(job)
}

/// Mark a job as failed. If attempts < max_attempts, reset to pending for retry.
pub async fn fail_job(
    pool: &PgPool,
    job_id: Uuid,
    error_result: Option<serde_json::Value>,
) -> Result<Job> {
    let job = sqlx::query_as::<_, Job>(
        r#"
        UPDATE jobs
        SET status = CASE
                WHEN attempts < max_attempts THEN 'pending'::job_status
                ELSE 'failed'::job_status
            END,
            result = $2,
            completed_at = CASE
                WHEN attempts >= max_attempts THEN now()
                ELSE NULL
            END
        WHERE id = $1 AND status = 'processing'
        RETURNING *
        "#,
    )
    .bind(job_id)
    .bind(&error_result)
    .fetch_optional(pool)
    .await?
    .ok_or(PipelineError::JobNotFound(job_id))?;

    Ok(job)
}

/// Get a job by ID.
pub async fn get_job(pool: &PgPool, job_id: Uuid) -> Result<Job> {
    let job = sqlx::query_as::<_, Job>(r#"SELECT * FROM jobs WHERE id = $1"#)
        .bind(job_id)
        .fetch_optional(pool)
        .await?
        .ok_or(PipelineError::JobNotFound(job_id))?;

    Ok(job)
}

/// List jobs with optional status filter.
pub async fn list_jobs(pool: &PgPool, status: Option<JobStatus>) -> Result<Vec<Job>> {
    let jobs = match status {
        Some(s) => {
            sqlx::query_as::<_, Job>(
                r#"SELECT * FROM jobs WHERE status = $1 ORDER BY priority DESC, created_at ASC"#,
            )
            .bind(s)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Job>(r#"SELECT * FROM jobs ORDER BY priority DESC, created_at ASC"#)
                .fetch_all(pool)
                .await?
        }
    };

    Ok(jobs)
}
