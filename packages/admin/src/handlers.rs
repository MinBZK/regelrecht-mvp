use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::{LawEntry, PaginatedResponse};

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
