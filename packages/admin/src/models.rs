use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct LawEntry {
    pub law_id: String,
    pub law_name: Option<String>,
    pub status: String,
    pub quality_score: Option<f64>,
    pub harvest_job_id: Option<sqlx::types::Uuid>,
    pub enrich_job_id: Option<sqlx::types::Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Job {
    pub id: sqlx::types::Uuid,
    pub job_type: String,
    pub law_id: String,
    pub status: String,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}
