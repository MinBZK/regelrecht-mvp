//! Integration tests for trajects that write to a local directory instead
//! of a GitHub repo: creating one from a local source in the registry, and
//! the git-backed "Bewerkt" list on top of it.
//!
//! Hermetic like `traject_reads_test.rs`: a Postgres container per test,
//! a temporary git repository as the corpus checkout, no GitHub.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use pretty_assertions::assert_eq;
use sqlx::PgPool;
use tokio::sync::{Mutex, RwLock};
use tower_sessions::Session;
use tower_sessions_memory_store::MemoryStore;
use uuid::Uuid;

use regelrecht_auth::handlers::{
    SESSION_KEY_EMAIL, SESSION_KEY_EMAIL_VERIFIED, SESSION_KEY_NAME, SESSION_KEY_SUB,
};
use regelrecht_corpus::models::{LocalSource, Source, SourceType};
use regelrecht_corpus::CorpusRegistry;
use regelrecht_editor_api::accounts::AccountRecord;
use regelrecht_editor_api::config::AppConfig;
use regelrecht_editor_api::corpus_handlers::{
    get_traject_annotations, get_traject_corpus_law, list_traject_changed_laws, save_law,
};
use regelrecht_editor_api::state::{AppState, CorpusState};
use regelrecht_editor_api::traject_corpus::TrajectCorpusCache;
use regelrecht_editor_api::trajects::{self, CreateTrajectRequest};

use regelrecht_pipeline::test_utils::TestDb;

const LAW_ID: &str = "wet_op_de_zorgtoeslag";
const LOCAL_SOURCE_ID: &str = "lokaal";
const ORIGINAL: &str = "$id: wet_op_de_zorgtoeslag\nname: origineel\n";

/// App state whose registry holds one local source at `corpus_dir`.
fn state_with_local_source(pool: PgPool, corpus_dir: &FsPath) -> AppState {
    state_with_local_source_in(pool, corpus_dir, None)
}

/// [`state_with_local_source`] with a repository root for the notes.
fn state_with_local_source_in(
    pool: PgPool,
    corpus_dir: &FsPath,
    repo_root: Option<&FsPath>,
) -> AppState {
    let mut corpus = CorpusState::empty();
    corpus.registry = CorpusRegistry::from_sources(vec![Source {
        id: LOCAL_SOURCE_ID.to_string(),
        name: "Lokale checkout".to_string(),
        source_type: SourceType::Local {
            local: LocalSource {
                path: corpus_dir.to_path_buf(),
                repo_root: repo_root.map(FsPath::to_path_buf),
            },
        },
        scopes: Vec::new(),
        priority: 1,
        auth_ref: None,
        strict_auth: false,
    }]);
    AppState {
        corpus: Arc::new(RwLock::new(corpus)),
        oidc_client: None,
        end_session_url: None,
        config: Arc::new(AppConfig {
            oidc: None,
            base_url: None,
            github_oauth: None,
            task_enrich_provider: "claude".to_string(),
        }),
        http_client: regelrecht_auth::http_client(),
        pool: Some(pool),
        pipeline_api_url: None,
        harvest_admin_url: None,
        reload_lock: Arc::new(Mutex::new(())),
        integrity: Default::default(),
        trajects: Arc::new(TrajectCorpusCache::new()),
    }
}

async fn seed_account(pool: &PgPool, email: &str) -> AccountRecord {
    let sub = format!("sub-{email}");
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO accounts (person_sub, email, name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&sub)
    .bind(email)
    .bind("Test User")
    .fetch_one(pool)
    .await
    .unwrap();
    AccountRecord {
        id,
        person_sub: sub,
        email: email.to_string(),
        name: "Test User".to_string(),
    }
}

async fn session_for(account: &AccountRecord) -> Session {
    let session = Session::new(None, Arc::new(MemoryStore::default()), None);
    session
        .insert(SESSION_KEY_SUB, &account.person_sub)
        .await
        .unwrap();
    session.insert(SESSION_KEY_NAME, "Test User").await.unwrap();
    session
        .insert(SESSION_KEY_EMAIL, &account.email)
        .await
        .unwrap();
    session
        .insert(SESSION_KEY_EMAIL_VERIFIED, true)
        .await
        .unwrap();
    session
}

fn git(dir: &FsPath, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}: {out:?}");
}

/// A git repository with one committed law under `corpus/`. Returns the
/// corpus directory, the source root of the local source.
fn corpus_checkout(repo: &FsPath) -> PathBuf {
    git(repo, &["init", "--quiet"]);
    git(repo, &["config", "user.name", "test"]);
    git(repo, &["config", "user.email", "test@test.local"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
    let corpus = repo.join("corpus");
    let law_dir = corpus.join("wet").join(LAW_ID);
    std::fs::create_dir_all(&law_dir).unwrap();
    std::fs::write(law_dir.join("2025-01-01.yaml"), ORIGINAL).unwrap();
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "--quiet", "--no-verify", "-m", "corpus"]);
    corpus
}

fn local_create_req(name: &str) -> CreateTrajectRequest {
    CreateTrajectRequest {
        name: name.to_string(),
        description: String::new(),
        scope: String::new(),
        repo_owner: None,
        repo_name: None,
        base_branch: None,
        repo_path: None,
        local_source_id: Some(LOCAL_SOURCE_ID.to_string()),
    }
}

/// Create a local traject through the handler and return `(id, ref)`.
async fn create_local_traject(state: &AppState, owner: &AccountRecord) -> (Uuid, String) {
    let (status, Json(summary)) = trajects::create(
        State(state.clone()),
        Extension(owner.clone()),
        HeaderMap::new(),
        Json(local_create_req("Lokaal werk")),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::CREATED);
    (summary.id, summary.traject_ref.unwrap())
}

async fn changed_laws(state: &AppState, owner: &AccountRecord, tref: &str) -> Vec<String> {
    let Json(ids) = list_traject_changed_laws(
        State(state.clone()),
        Extension(owner.clone()),
        session_for(owner).await,
        Path(tref.to_string()),
        HeaderMap::new(),
    )
    .await
    .unwrap();
    ids
}

async fn read_law(state: &AppState, owner: &AccountRecord, tref: &str) -> String {
    let (status, _, body) = get_traject_corpus_law(
        State(state.clone()),
        Extension(owner.clone()),
        session_for(owner).await,
        Path((tref.to_string(), LAW_ID.to_string())),
        HeaderMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    body
}

#[tokio::test]
async fn local_targets_list_the_local_sources() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    let state = state_with_local_source(db.pool.clone(), repo.path());

    let Json(targets) = trajects::list_local_targets(State(state)).await;
    let ids: Vec<&str> = targets.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec![LOCAL_SOURCE_ID]);
}

#[tokio::test]
async fn create_with_a_local_target_writes_to_that_directory() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    let corpus = corpus_checkout(repo.path());
    let state = state_with_local_source(db.pool.clone(), &corpus);
    let alice = seed_account(&db.pool, "alice@test.local").await;

    let (traject_id, _) = create_local_traject(&state, &alice).await;

    let rows: Vec<(String, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT source_id, source_type::text, local_path, is_writable_own
         FROM traject_corpus_sources WHERE traject_id = $1 ORDER BY source_id",
    )
    .bind(traject_id)
    .fetch_all(&db.pool)
    .await
    .unwrap();
    // One row: the local directory as the writable-own source. The seed
    // with the same directory is not added next to it.
    assert_eq!(
        rows,
        vec![(
            format!("traject-own-{}", traject_id.simple()),
            "local".to_string(),
            Some(corpus.to_string_lossy().to_string()),
            true,
        )]
    );
}

#[tokio::test]
async fn create_refuses_a_local_target_that_does_not_exist() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    let state = state_with_local_source(db.pool.clone(), &repo.path().join("bestaat-niet"));
    let alice = seed_account(&db.pool, "alice@test.local").await;

    let (status, _) = trajects::create(
        State(state),
        Extension(alice),
        HeaderMap::new(),
        Json(local_create_req("Nergens")),
    )
    .await
    .unwrap_err();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_refuses_an_unknown_local_target_or_one_mixed_with_a_repo() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    let state = state_with_local_source(db.pool.clone(), repo.path());
    let alice = seed_account(&db.pool, "alice@test.local").await;

    let mut unknown = local_create_req("Onbekend");
    unknown.local_source_id = Some("bestaat-niet".to_string());
    let mut mixed = local_create_req("Allebei");
    mixed.repo_owner = Some("example-org".to_string());

    for req in [unknown, mixed] {
        let (status, _) = trajects::create(
            State(state.clone()),
            Extension(alice.clone()),
            HeaderMap::new(),
            Json(req),
        )
        .await
        .unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trajects")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn a_saved_law_shows_as_changed_until_git_restores_it() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    let corpus = corpus_checkout(repo.path());
    let state = state_with_local_source(db.pool.clone(), &corpus);
    let alice = seed_account(&db.pool, "alice@test.local").await;
    let (traject_id, tref) = create_local_traject(&state, &alice).await;
    assert!(changed_laws(&state, &alice, &tref).await.is_empty());

    let edited = format!("$id: {LAW_ID}\nname: bewerkt\n");
    save_law(
        State(state.clone()),
        Extension(alice.clone()),
        session_for(&alice).await,
        Path((tref.clone(), LAW_ID.to_string())),
        HeaderMap::new(),
        edited.clone(),
    )
    .await
    .unwrap();
    let law_file = corpus.join("wet").join(LAW_ID).join("2025-01-01.yaml");
    assert_eq!(std::fs::read_to_string(&law_file).unwrap(), edited);
    assert_eq!(read_law(&state, &alice, &tref).await, edited);
    // From git, not from the save's own bookkeeping: drop the cache first.
    state.trajects.invalidate(traject_id).await;
    assert_eq!(changed_laws(&state, &alice, &tref).await, vec![LAW_ID]);

    // Undo is git's job: once the file is back at the committed version the
    // diff is empty again.
    std::process::Command::new("git")
        .args(["restore", "--", "."])
        .current_dir(&corpus)
        .output()
        .unwrap();
    assert_eq!(std::fs::read_to_string(&law_file).unwrap(), ORIGINAL);
    // Stands in for the changed-laws cache expiring (one minute).
    state.trajects.invalidate(traject_id).await;
    assert!(changed_laws(&state, &alice, &tref).await.is_empty());
}

#[tokio::test]
async fn a_local_traject_reads_notes_from_the_repo_root() {
    let db = TestDb::new().await;
    let repo = tempfile::TempDir::new().unwrap();
    // The monorepo layout: `corpus/regulation/nl` holds the laws and
    // `corpus/annotations` the notes, next to it rather than inside.
    let root = repo.path().join("corpus");
    let regulation = root.join("regulation").join("nl");
    let law_dir = regulation.join("wet").join(LAW_ID);
    std::fs::create_dir_all(&law_dir).unwrap();
    std::fs::write(law_dir.join("2025-01-01.yaml"), ORIGINAL).unwrap();
    let notes_dir = root.join("annotations").join(LAW_ID);
    std::fs::create_dir_all(&notes_dir).unwrap();
    let notes = "annotations: []\n";
    std::fs::write(notes_dir.join("annotations.yaml"), notes).unwrap();

    let state = state_with_local_source_in(db.pool.clone(), &regulation, Some(&root));
    let alice = seed_account(&db.pool, "alice@test.local").await;
    let (traject_id, tref) = create_local_traject(&state, &alice).await;

    let (stored,): (Option<String>,) = sqlx::query_as(
        "SELECT local_repo_root FROM traject_corpus_sources
         WHERE traject_id = $1 AND is_writable_own",
    )
    .bind(traject_id)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(stored, Some(root.to_string_lossy().to_string()));

    let (status, _, body) = get_traject_annotations(
        State(state.clone()),
        session_for(&alice).await,
        Extension(alice.clone()),
        Path((tref, LAW_ID.to_string())),
        HeaderMap::new(),
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, notes);
}
