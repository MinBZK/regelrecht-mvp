use tracing_subscriber::EnvFilter;

use regelrecht_pipeline::config::WorkerConfig;
use regelrecht_pipeline::worker::run_harvest_worker;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = match WorkerConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!(error = %e, "failed to load configuration");
            std::process::exit(1);
        }
    };

    if let Err(e) = run_harvest_worker(config).await {
        tracing::error!(error = %e, "harvest worker exited with error");
        std::process::exit(1);
    }
}
