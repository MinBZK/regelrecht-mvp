//! De server van de poc. Zie de crate-documentatie in `lib.rs`.

use std::path::PathBuf;

use regelrecht_poc_vakantieverlof::{api::AppState, kern::Corpus, router};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "regelrecht_poc_vakantieverlof=info,tower_http=info".into()),
        )
        .init();

    // De repository-map `corpus/`: de regelwerken onder regulation/nl en de
    // cellen onder cells/.
    let corpus_dir = std::env::var("VERLOF_CORPUS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus")));
    // De casussen: per casus de cel, het artikel en de tekst voor de bezoeker.
    let casus_dir = std::env::var("VERLOF_CASUS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/casussen")));
    let state = AppState::new(Corpus::alle(&corpus_dir, &casus_dir)?);

    let base = std::env::var("VERLOF_BASE_PATH").unwrap_or_else(|_| "/".into());
    let static_dir = std::env::var("VERLOF_STATIC_DIR").unwrap_or_else(|_| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../frontend-poc-vakantieverlof/dist"
        )
        .into()
    });
    let app = router(state, &base, &static_dir).layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("VERLOF_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8410);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%base, "poc luistert op http://localhost:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}
