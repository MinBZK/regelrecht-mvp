use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::{Job, LawEntry, PaginatedResponse};

#[derive(Deserialize)]
pub struct LawEntriesQuery {
    pub status: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

const ALLOWED_SORT_COLUMNS_LAW: &[&str] = &[
    "law_id",
    "law_name",
    "status",
    "quality_score",
    "created_at",
    "updated_at",
];

pub async fn list_law_entries(
    State(pool): State<PgPool>,
    Query(params): Query<LawEntriesQuery>,
) -> Result<Json<PaginatedResponse<LawEntry>>, StatusCode> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let sort_column = params.sort.as_deref().unwrap_or("updated_at");
    if !ALLOWED_SORT_COLUMNS_LAW.contains(&sort_column) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let order = match params.order.as_deref() {
        Some("ASC" | "asc") => "ASC",
        _ => "DESC",
    };

    // Count query
    let total: i64 = if let Some(ref status) = params.status {
        sqlx::query_scalar("SELECT COUNT(*) FROM law_entries WHERE status::text = $1")
            .bind(status)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "count query failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM law_entries")
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "count query failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };

    // Data query — sort column is validated against an allowlist above, so
    // interpolating it into the query string is safe.
    let query_str = if params.status.is_some() {
        format!(
            "SELECT law_id, law_name, status::text as status, quality_score, \
             harvest_job_id, enrich_job_id, created_at, updated_at \
             FROM law_entries WHERE status::text = $1 \
             ORDER BY {sort_column} {order} LIMIT $2 OFFSET $3"
        )
    } else {
        format!(
            "SELECT law_id, law_name, status::text as status, quality_score, \
             harvest_job_id, enrich_job_id, created_at, updated_at \
             FROM law_entries \
             ORDER BY {sort_column} {order} LIMIT $1 OFFSET $2"
        )
    };

    let data: Vec<LawEntry> = if let Some(ref status) = params.status {
        sqlx::query_as::<_, LawEntry>(&query_str)
            .bind(status)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "data query failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else {
        sqlx::query_as::<_, LawEntry>(&query_str)
            .bind(limit)
            .bind(offset)
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "data query failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };

    Ok(Json(PaginatedResponse {
        data,
        total,
        limit,
        offset,
    }))
}

// --- Jobs ---

#[derive(Deserialize)]
pub struct JobsQuery {
    pub status: Option<String>,
    pub job_type: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

const ALLOWED_SORT_COLUMNS_JOB: &[&str] = &[
    "id",
    "job_type",
    "law_id",
    "status",
    "priority",
    "attempts",
    "created_at",
    "updated_at",
    "started_at",
    "completed_at",
];

pub async fn list_jobs(
    State(pool): State<PgPool>,
    Query(params): Query<JobsQuery>,
) -> Result<Json<PaginatedResponse<Job>>, StatusCode> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let sort_column = params.sort.as_deref().unwrap_or("created_at");
    if !ALLOWED_SORT_COLUMNS_JOB.contains(&sort_column) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let order = match params.order.as_deref() {
        Some("ASC" | "asc") => "ASC",
        _ => "DESC",
    };

    // Build dynamic WHERE clause for dual-filter support.
    let mut where_clauses = Vec::new();
    let mut bind_index: usize = 1;

    if params.status.is_some() {
        where_clauses.push(format!("status::text = ${bind_index}"));
        bind_index += 1;
    }

    if params.job_type.is_some() {
        where_clauses.push(format!("job_type::text = ${bind_index}"));
        bind_index += 1;
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count query
    let count_sql = format!("SELECT COUNT(*) FROM jobs {where_sql}");

    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(ref status) = params.status {
        count_query = count_query.bind(status);
    }
    if let Some(ref job_type) = params.job_type {
        count_query = count_query.bind(job_type);
    }

    let total: i64 = count_query.fetch_one(&pool).await.map_err(|e| {
        tracing::error!(error = %e, "count query failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Data query — sort column is validated against an allowlist above, so
    // interpolating it into the query string is safe.
    let limit_idx = bind_index;
    let offset_idx = bind_index + 1;

    let data_sql = format!(
        "SELECT id, job_type::text as job_type, law_id, status::text as status, \
         priority, attempts, max_attempts, created_at, updated_at, started_at, completed_at \
         FROM jobs {where_sql} \
         ORDER BY {sort_column} {order} LIMIT ${limit_idx} OFFSET ${offset_idx}"
    );

    let mut data_query = sqlx::query_as::<_, Job>(&data_sql);
    if let Some(ref status) = params.status {
        data_query = data_query.bind(status);
    }
    if let Some(ref job_type) = params.job_type {
        data_query = data_query.bind(job_type);
    }
    data_query = data_query.bind(limit).bind(offset);

    let data: Vec<Job> = data_query.fetch_all(&pool).await.map_err(|e| {
        tracing::error!(error = %e, "data query failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(PaginatedResponse {
        data,
        total,
        limit,
        offset,
    }))
}
