use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

mod handlers;
mod models;

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 10;
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

async fn health() -> &'static str {
    "OK"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let database_url = match env::var("DATABASE_SERVER_FULL") {
        Ok(url) => url,
        Err(_) => {
            tracing::error!("DATABASE_SERVER_FULL environment variable is not set");
            std::process::exit(1);
        }
    };

    tracing::info!("connecting to database...");

    let mut pool = None;
    for attempt in 1..=MAX_RETRIES {
        match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(ACQUIRE_TIMEOUT)
            .connect(&database_url)
            .await
        {
            Ok(p) => {
                pool = Some(p);
                break;
            }
            Err(e) => {
                tracing::warn!(attempt, error = %e, "failed to connect, retrying...");
                if attempt == MAX_RETRIES {
                    tracing::error!("exhausted all {MAX_RETRIES} connection attempts");
                    std::process::exit(1);
                }
                tokio::time::sleep(RETRY_INTERVAL).await;
            }
        }
    }
    let pool = pool.unwrap_or_else(|| {
        tracing::error!("unreachable: no pool after retry loop");
        std::process::exit(1);
    });

    tracing::info!("connected to database");

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/law_entries", get(handlers::list_law_entries))
        .route("/api/jobs", get(handlers::list_jobs))
        .with_state(pool)
        .fallback_service(ServeDir::new("static"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "failed to bind on {addr}");
        std::process::exit(1);
    });

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!(error = %e, "server error");
        std::process::exit(1);
    }
}
