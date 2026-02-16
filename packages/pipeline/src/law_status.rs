use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{PipelineError, Result};
use crate::models::{LawEntry, LawStatusValue};

/// Upsert a law entry. Creates it if it doesn't exist, updates name if it does.
pub async fn upsert_law(pool: &PgPool, law_id: &str, law_name: Option<&str>) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(
        r#"
        INSERT INTO law_entries (law_id, law_name)
        VALUES ($1, $2)
        ON CONFLICT (law_id) DO UPDATE SET law_name = COALESCE($2, law_entries.law_name)
        RETURNING *
        "#,
    )
    .bind(law_id)
    .bind(law_name)
    .fetch_one(pool)
    .await?;

    Ok(entry)
}

/// Update the status of a law entry.
pub async fn update_status(
    pool: &PgPool,
    law_id: &str,
    status: LawStatusValue,
) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(
        r#"
        UPDATE law_entries SET status = $2
        WHERE law_id = $1
        RETURNING *
        "#,
    )
    .bind(law_id)
    .bind(status)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PipelineError::LawNotFound(law_id.to_string()))?;

    Ok(entry)
}

/// Link a harvest job to a law entry.
pub async fn set_harvest_job(pool: &PgPool, law_id: &str, job_id: Uuid) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(
        r#"
        UPDATE law_entries SET harvest_job_id = $2
        WHERE law_id = $1
        RETURNING *
        "#,
    )
    .bind(law_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PipelineError::LawNotFound(law_id.to_string()))?;

    Ok(entry)
}

/// Link an enrich job to a law entry.
pub async fn set_enrich_job(pool: &PgPool, law_id: &str, job_id: Uuid) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(
        r#"
        UPDATE law_entries SET enrich_job_id = $2
        WHERE law_id = $1
        RETURNING *
        "#,
    )
    .bind(law_id)
    .bind(job_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PipelineError::LawNotFound(law_id.to_string()))?;

    Ok(entry)
}

/// Set the quality score for a law entry.
pub async fn set_quality_score(pool: &PgPool, law_id: &str, score: f64) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(
        r#"
        UPDATE law_entries SET quality_score = $2
        WHERE law_id = $1
        RETURNING *
        "#,
    )
    .bind(law_id)
    .bind(score)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PipelineError::LawNotFound(law_id.to_string()))?;

    Ok(entry)
}

/// Get a law entry by ID.
pub async fn get_law(pool: &PgPool, law_id: &str) -> Result<LawEntry> {
    let entry = sqlx::query_as::<_, LawEntry>(r#"SELECT * FROM law_entries WHERE law_id = $1"#)
        .bind(law_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PipelineError::LawNotFound(law_id.to_string()))?;

    Ok(entry)
}

/// List all law entries, optionally filtered by status.
pub async fn list_laws(pool: &PgPool, status: Option<LawStatusValue>) -> Result<Vec<LawEntry>> {
    let entries = match status {
        Some(s) => {
            sqlx::query_as::<_, LawEntry>(
                r#"SELECT * FROM law_entries WHERE status = $1 ORDER BY law_id"#,
            )
            .bind(s)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, LawEntry>(r#"SELECT * FROM law_entries ORDER BY law_id"#)
                .fetch_all(pool)
                .await?
        }
    };

    Ok(entries)
}
