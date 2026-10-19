//! PoC: een uitvoeringsproces naspelen op de cel van een fictieve uitvoerder
//! (chronolexografie, RFC-045 tot en met RFC-050). De bezoeker kiest een
//! casus: vakantieverlof (Leerplichtwet 1969 art. 13a) bij OBS De Klimop, of
//! de gehandicaptenparkeerkaart (BABW art. 49) bij de gemeente Wilgendaal.
//!
//! De cel zelf staat in `packages/cel`; deze crate zet er een kleine API voor
//! een app omheen ([`api`]) en voegt niets toe aan wat de cel doet ([`kern`]).
//! Een casus noemt alleen de cel en het artikel dat de aanvraag vestigt; de
//! rest leidt [`casus`] af uit de wet.

pub mod api;
pub mod casus;
pub mod kern;

use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

/// The app under `base` (`/` or `/<prefix>/`): the API, and the built
/// frontend in `static_dir` with its single-page fallback.
///
/// The prefix goes in the router, not in a layer that rewrites the URI: axum
/// matches the path before a layer runs (see the `poc-add` skill). `nest`
/// does not route the bare `/<prefix>/` to the nested root, so that one
/// redirects to `/<prefix>`.
pub fn router(state: api::AppState, base: &str, static_dir: &str) -> Router {
    let index = format!("{static_dir}/index.html");
    let inner = Router::new()
        .route_service("/", ServeFile::new(&index))
        .route("/health", get(|| async { "OK" }))
        .merge(api::routes())
        .with_state(state)
        .fallback_service(ServeDir::new(static_dir).not_found_service(ServeFile::new(&index)));
    let prefix = base.trim_matches('/');
    if prefix.is_empty() {
        return inner;
    }
    let prefix = format!("/{prefix}");
    let naar = prefix.clone();
    Router::new()
        .route(
            &format!("{prefix}/"),
            get(move || async move { Redirect::permanent(&naar) }),
        )
        .nest(&prefix, inner)
}
