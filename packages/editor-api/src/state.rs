use std::sync::Arc;

use regelrecht_corpus::SourceMap;
use sqlx::PgPool;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    /// Loaded corpus sources with provenance metadata.
    pub corpus: Arc<RwLock<CorpusState>>,
    /// Optional database connection for feature flags.
    pub pool: Option<PgPool>,
}

/// State for the corpus subsystem.
pub struct CorpusState {
    pub registry: regelrecht_corpus::CorpusRegistry,
    pub source_map: SourceMap,
}
