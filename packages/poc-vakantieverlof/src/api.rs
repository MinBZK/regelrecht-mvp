//! The HTTP API under `/api`: one handler per step of [`crate::kern`].
//!
//! The app plays one case at a time (a casus: a cell and the article that
//! establishes the application). `GET /api/casussen` lists them, `POST
//! /api/casus` chooses one and starts its cell with an empty chronicle, and
//! every other endpoint works on the chosen case; without a choice it says
//! so (409). The state is in memory and shared by every visitor; `POST
//! /api/reset` empties the chronicle of the chosen case. That is enough for a
//! PoC that plays the process through and loses nothing worth keeping on a
//! restart.

use std::sync::{Arc, Mutex, MutexGuard};

use axum::extract::{FromRequestParts, Path, Query, State};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use regelrecht_cel::{Cell, Error};
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::kern::{self, Corpus};

/// The chosen case and its cell.
struct Spel {
    casus: usize,
    cell: Cell,
}

#[derive(Clone)]
pub struct AppState {
    casussen: Arc<Vec<Corpus>>,
    spel: Arc<Mutex<Option<Spel>>>,
}

impl AppState {
    /// The app with `casussen` to choose from, and none chosen yet.
    pub fn new(casussen: Vec<Corpus>) -> Self {
        Self {
            casussen: Arc::new(casussen),
            spel: Arc::new(Mutex::new(None)),
        }
    }

    /// The chosen case and its cell. A step that panicked halfway leaves the
    /// chronicle as the cell left it: it appends whole or not at all.
    fn spel(&self) -> MutexGuard<'_, Option<Spel>> {
        self.spel.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Run `stap` on the chosen case and its cell. A request that names the
    /// case it expects (`X-Casus`) is refused when another visitor has chosen
    /// another case since: the step would land in the wrong cell.
    fn met<T>(
        &self,
        verwacht: &Verwacht,
        stap: impl FnOnce(&Corpus, &mut Cell) -> kern::Result<T>,
    ) -> Result<T, Fout> {
        let mut spel = self.spel();
        let spel = spel.as_mut().ok_or(Fout::GeenCasus)?;
        let corpus = &self.casussen[spel.casus];
        if let Some(id) = verwacht.0.as_deref() {
            if id != corpus.casus.id {
                return Err(Fout::AndereCasus {
                    verwacht: id.to_string(),
                    gekozen: corpus.casus.id.clone(),
                });
            }
        }
        Ok(stap(corpus, &mut spel.cell)?)
    }
}

/// The case a request expects, from its `X-Casus` header. The app sends it
/// with every step; a request without it works on whatever case is chosen.
pub struct Verwacht(Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Verwacht {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .headers
                .get("x-casus")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string),
        ))
    }
}

/// The routes under `/api`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/casussen", get(casussen))
        .route("/api/casus", get(casus).post(kiezen))
        .route("/api/vormen", get(vormen))
        .route("/api/kroniek", get(kroniek))
        .route("/api/aanvragen", post(aanvragen))
        .route("/api/besluiten", post(besluiten))
        .route("/api/besluiten/{id}/waarom", get(waarom))
        .route("/api/besluiten/{id}/antwoord/{begrip}", get(hoe_beantwoord))
        .route("/api/bezwaren", post(bezwaren))
        .route("/api/bezwaren/{id}/heroverweging", get(heroverweging))
        .route("/api/beslissingen-op-bezwaar", post(beslissen_op_bezwaar))
        .route("/api/reducties", get(reducties))
        .route("/api/logboek", get(logboek))
        .route("/api/reset", post(reset))
}

/// An error as the frontend reads it: `{ "fout": ... }`. What the cell or
/// the engine refuses is the visitor's to correct (422), as is a step before
/// a case is chosen or after another case was chosen (409), or a case that
/// does not exist (404). A chronicle that cannot be written, or an answer
/// that cannot be serialized, is ours (500).
enum Fout {
    Cel(Error),
    /// No case is chosen yet (409).
    GeenCasus,
    /// There is no case by that id (404).
    OnbekendeCasus(String),
    /// Another case was chosen since the request's own (409).
    AndereCasus {
        verwacht: String,
        gekozen: String,
    },
    Intern(String),
}

impl From<Error> for Fout {
    fn from(e: Error) -> Self {
        Self::Cel(e)
    }
}

impl IntoResponse for Fout {
    fn into_response(self) -> Response {
        let (status, fout) = match self {
            Self::Cel(e @ Error::Io(_)) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Cel(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()),
            Self::GeenCasus => (
                StatusCode::CONFLICT,
                "no case is chosen: choose one with POST /api/casus".to_string(),
            ),
            Self::OnbekendeCasus(id) => (StatusCode::NOT_FOUND, format!("there is no case '{id}'")),
            Self::AndereCasus { verwacht, gekozen } => (
                StatusCode::CONFLICT,
                format!("the chosen case is now '{gekozen}', not '{verwacht}'"),
            ),
            Self::Intern(e) => (StatusCode::INTERNAL_SERVER_ERROR, e),
        };
        (status, Json(json!({ "fout": fout }))).into_response()
    }
}

type Antwoord = std::result::Result<Json<Value>, Fout>;

fn ok(value: impl serde::Serialize) -> Antwoord {
    serde_json::to_value(value)
        .map(Json)
        .map_err(|e| Fout::Intern(e.to_string()))
}

#[derive(Deserialize)]
struct Dag {
    datum: NaiveDate,
}

/// The cases to choose from, and which is chosen.
async fn casussen(State(s): State<AppState>) -> Antwoord {
    let gekozen = s
        .spel()
        .as_ref()
        .map(|g| s.casussen[g.casus].casus.id.clone());
    let kaarten: Vec<Value> = s.casussen.iter().map(Corpus::kaart).collect();
    ok(json!({ "casussen": kaarten, "gekozen": gekozen }))
}

/// The chosen case, as the app shows it.
async fn casus(State(s): State<AppState>, v: Verwacht) -> Antwoord {
    ok(s.met(&v, |c, _| Ok(c.beschrijving()))?)
}

#[derive(Deserialize)]
struct Keuze {
    casus: String,
}

/// Choose a case: its cell starts with an empty chronicle.
async fn kiezen(State(s): State<AppState>, Json(k): Json<Keuze>) -> Antwoord {
    let i = s
        .casussen
        .iter()
        .position(|c| c.casus.id == k.casus)
        .ok_or_else(|| Fout::OnbekendeCasus(k.casus.clone()))?;
    let corpus = &s.casussen[i];
    let cell = corpus.cell()?;
    *s.spel() = Some(Spel { casus: i, cell });
    ok(corpus.beschrijving())
}

async fn vormen(State(s): State<AppState>, v: Verwacht, Query(q): Query<Dag>) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::vormen(c, cell, q.datum))?)
}

async fn kroniek(State(s): State<AppState>, v: Verwacht) -> Antwoord {
    ok(s.met(&v, |_, cell| {
        Ok(json!({ "cel": cell.recording_actor(), "grammen": cell.grams() }))
    })?)
}

#[derive(Deserialize)]
struct Indiening {
    #[serde(default)]
    velden: Map<String, Value>,
    datum: NaiveDate,
}

async fn aanvragen(State(s): State<AppState>, v: Verwacht, Json(b): Json<Indiening>) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::aanvragen(c, cell, &b.velden, b.datum))?)
}

#[derive(Deserialize)]
struct Beslissing {
    /// The application, or the objection, decided on.
    op: String,
    /// The deciding authority's findings for this case (RFC-050).
    #[serde(default)]
    bevindingen: Map<String, Value>,
    datum: NaiveDate,
}

async fn besluiten(State(s): State<AppState>, v: Verwacht, Json(b): Json<Beslissing>) -> Antwoord {
    ok(s.met(&v, |c, cell| {
        kern::besluiten(c, cell, &b.op, &b.bevindingen, b.datum)
    })?)
}

async fn waarom(State(s): State<AppState>, v: Verwacht, Path(id): Path<String>) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::waarom(c, cell, &id))?)
}

async fn hoe_beantwoord(
    State(s): State<AppState>,
    v: Verwacht,
    Path((id, begrip)): Path<(String, String)>,
) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::hoe_beantwoord(c, cell, &id, &begrip))?)
}

#[derive(Deserialize)]
struct Bezwaar {
    besluit: String,
    #[serde(default)]
    velden: Map<String, Value>,
    datum: NaiveDate,
}

async fn bezwaren(State(s): State<AppState>, v: Verwacht, Json(b): Json<Bezwaar>) -> Antwoord {
    ok(s.met(&v, |c, cell| {
        kern::bezwaren(c, cell, &b.besluit, &b.velden, b.datum)
    })?)
}

async fn heroverweging(
    State(s): State<AppState>,
    v: Verwacht,
    Path(id): Path<String>,
    Query(q): Query<Dag>,
) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::heroverweging(c, cell, &id, q.datum))?)
}

async fn beslissen_op_bezwaar(
    State(s): State<AppState>,
    v: Verwacht,
    Json(b): Json<Beslissing>,
) -> Antwoord {
    ok(s.met(&v, |c, cell| {
        kern::beslissen_op_bezwaar(c, cell, &b.op, &b.bevindingen, b.datum)
    })?)
}

async fn reducties(State(s): State<AppState>, v: Verwacht, Query(q): Query<Dag>) -> Antwoord {
    ok(s.met(&v, |c, cell| kern::reducties(c, cell, q.datum))?)
}

async fn logboek(State(s): State<AppState>, v: Verwacht) -> Antwoord {
    ok(s.met(&v, |c, cell| {
        Ok(json!({ "handelingen": kern::logboek(c, cell)? }))
    })?)
}

async fn reset(State(s): State<AppState>, v: Verwacht) -> Antwoord {
    s.met(&v, |c, cell| {
        *cell = c.cell()?;
        Ok(())
    })?;
    ok(json!({ "grammen": [] }))
}
