use std::env;
use std::io::Write;
use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

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

    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!(error = %e, "failed to connect to database");
            std::process::exit(1);
        }
    };

    tracing::info!("running migrations...");

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::error!(error = %e, "failed to run migrations");
        std::process::exit(1);
    }

    tracing::info!("migrations completed successfully");

    // Serve minimal health endpoint on port 8000 (required by RIG liveprobe).
    let listener = match TcpListener::bind("0.0.0.0:8000") {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(error = %e, "failed to bind health endpoint on port 8000");
            std::process::exit(1);
        }
    };

    tracing::info!("health endpoint listening on :8000");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK");
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to accept connection");
            }
        }
    }
}
