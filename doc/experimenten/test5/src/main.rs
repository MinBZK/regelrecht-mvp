//! HighlightEditor Rust Server (Axum)
//!
//! Migrated from server.py - serves the RegelRecht annotation browser.
//!
//! Endpoints:
//! - GET  /api/regulations           → list all regulations
//! - GET  /api/regulation/{id}       → regulation + annotations
//! - POST /api/regulation/{id}/annotation → save W3C annotation
//! - GET  /api/regulation/{id}/annotations → get annotations only
//! - Static files from frontend/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};

// Custom deserializer that accepts both strings and numbers as String
fn string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;
    Ok(value.map(|v| match v {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    }))
}
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{info, warn};
use walkdir::WalkDir;

// =============================================================================
// Configuration
// =============================================================================

const MAX_BODY_SIZE: usize = 10 * 1024; // 10KB max request body

// W3C Web Annotation vocabulary - valid motivations
const VALID_MOTIVATIONS: &[&str] = &[
    "assessing",
    "bookmarking",
    "classifying",
    "commenting",
    "describing",
    "editing",
    "highlighting",
    "identifying",
    "linking",
    "moderating",
    "questioning",
    "replying",
    "tagging",
];

const VALID_RESOLUTIONS: &[&str] = &["found", "orphaned"];
const VALID_WORKFLOWS: &[&str] = &["open", "resolved"];
const VALID_CLASSIFICATIONS: &[&str] = &[
    "definition",
    "input",
    "output",
    "logic",
    "open_norm",
    "parameter",
];
const VALID_DATA_TYPES: &[&str] = &[
    "boolean", "integer", "number", "string", "date", "money", "amount",
];

// =============================================================================
// Types
// =============================================================================

#[derive(Clone)]
struct AppState {
    regulations: Arc<RwLock<HashMap<String, RegulationInfo>>>,
    regulation_dir: PathBuf,
    annotations_dir: PathBuf,
    frontend_dir: PathBuf,
}

#[derive(Clone, Debug)]
struct RegulationInfo {
    path: String,
    data: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct RegulationListItem {
    id: String,
    name: String,
    regulatory_layer: String,
    publication_date: Option<String>,
    valid_from: Option<String>,
    bwb_id: Option<String>,
    url: Option<String>,
    article_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnnotationRequest {
    #[serde(rename = "type")]
    annotation_type: String,
    motivation: String,
    #[serde(default)]
    resolution: Option<String>,
    #[serde(default)]
    workflow: Option<String>,
    target: AnnotationTarget,
    body: AnnotationBody,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnnotationTarget {
    #[serde(default)]
    source: Option<String>,
    #[serde(default, deserialize_with = "string_or_number")]
    article: Option<String>,
    #[serde(default)]
    selector: Option<AnnotationSelector>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnnotationSelector {
    #[serde(rename = "type")]
    selector_type: Option<String>,
    exact: Option<String>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    suffix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnnotationBody {
    #[serde(rename = "type")]
    body_type: String,
    #[serde(default)]
    purpose: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    classification: Option<String>,
    #[serde(default)]
    data_type: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    rule_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

// =============================================================================
// Validation
// =============================================================================

fn validate_regulation_id(reg_id: &str) -> bool {
    if reg_id.is_empty() || reg_id.len() > 100 {
        return false;
    }
    let re = Regex::new(r"^[a-z0-9_]+$").unwrap();
    re.is_match(reg_id)
}

fn validate_annotation(annotation: &AnnotationRequest) -> Result<(), String> {
    // Validate type
    if annotation.annotation_type != "Annotation" {
        return Err("Invalid type: must be 'Annotation'".to_string());
    }

    // Validate motivation
    if !VALID_MOTIVATIONS.contains(&annotation.motivation.as_str()) {
        return Err(format!("Invalid motivation: {}", annotation.motivation));
    }

    // Validate resolution if present
    if let Some(ref resolution) = annotation.resolution {
        if !VALID_RESOLUTIONS.contains(&resolution.as_str()) {
            return Err(format!("Invalid resolution: {}", resolution));
        }
    }

    // Validate workflow if present
    if let Some(ref workflow) = annotation.workflow {
        if !VALID_WORKFLOWS.contains(&workflow.as_str()) {
            return Err(format!("Invalid workflow: {}", workflow));
        }
    }

    // Validate target
    if annotation.target.source.is_none() {
        return Err("Target missing source".to_string());
    }

    // Validate selector if present
    if let Some(ref selector) = annotation.target.selector {
        if let Some(ref selector_type) = selector.selector_type {
            if selector_type != "TextQuoteSelector" {
                return Err("Invalid selector type: must be 'TextQuoteSelector'".to_string());
            }
        }
        if selector.exact.is_none() {
            return Err("Selector missing exact text".to_string());
        }
        if let Some(ref exact) = selector.exact {
            if exact.len() > 500 {
                return Err("Selector exact text too long (max 500 chars)".to_string());
            }
        }
    }

    // Validate body type
    if annotation.body.body_type != "TextualBody" && annotation.body.body_type != "SpecificResource"
    {
        return Err(format!(
            "Invalid body type: {} (must be TextualBody or SpecificResource)",
            annotation.body.body_type
        ));
    }

    // Validate body purpose if present
    if let Some(ref purpose) = annotation.body.purpose {
        if !VALID_MOTIVATIONS.contains(&purpose.as_str()) {
            return Err(format!("Invalid body purpose: {}", purpose));
        }
    }

    // Validate classification if present
    if let Some(ref classification) = annotation.body.classification {
        if !VALID_CLASSIFICATIONS.contains(&classification.as_str()) {
            return Err(format!("Invalid classification: {}", classification));
        }
    }

    // Validate data_type if present
    if let Some(ref data_type) = annotation.body.data_type {
        if !VALID_DATA_TYPES.contains(&data_type.as_str()) {
            return Err(format!("Invalid data_type: {}", data_type));
        }
    }

    Ok(())
}

// =============================================================================
// Data Loading
// =============================================================================

fn load_all_regulations(regulation_dir: &PathBuf) -> HashMap<String, RegulationInfo> {
    let mut regulations = HashMap::new();

    for entry in WalkDir::new(regulation_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "yaml"))
    {
        let path = entry.path();
        match fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str::<Value>(&content) {
                Ok(data) => {
                    if let Some(id) = data.get("$id").and_then(|v| v.as_str()) {
                        let relative_path = path
                            .strip_prefix(regulation_dir)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string();
                        regulations.insert(
                            id.to_string(),
                            RegulationInfo {
                                path: relative_path,
                                data,
                            },
                        );
                    }
                }
                Err(e) => warn!("Failed to parse YAML {}: {}", path.display(), e),
            },
            Err(e) => warn!("Failed to read {}: {}", path.display(), e),
        }
    }

    regulations
}

fn load_annotations(annotations_dir: &PathBuf, reg_id: &str) -> Vec<Value> {
    let annotations_file = annotations_dir.join(format!("{}.yaml", reg_id));
    if !annotations_file.exists() {
        return Vec::new();
    }

    match fs::read_to_string(&annotations_file) {
        Ok(content) => match serde_yaml::from_str::<Value>(&content) {
            Ok(data) => data
                .get("annotations")
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default(),
            Err(e) => {
                warn!("Failed to parse annotations {}: {}", annotations_file.display(), e);
                Vec::new()
            }
        },
        Err(e) => {
            warn!("Failed to read annotations {}: {}", annotations_file.display(), e);
            Vec::new()
        }
    }
}

fn save_annotations(annotations_dir: &PathBuf, reg_id: &str, annotations: &[Value]) -> Result<(), String> {
    fs::create_dir_all(annotations_dir).map_err(|e| format!("Failed to create annotations dir: {}", e))?;

    let annotations_file = annotations_dir.join(format!("{}.yaml", reg_id));
    let data = json!({ "annotations": annotations });

    let yaml_content = serde_yaml::to_string(&data)
        .map_err(|e| format!("Failed to serialize YAML: {}", e))?;

    fs::write(&annotations_file, yaml_content)
        .map_err(|e| format!("Failed to write annotations file: {}", e))?;

    Ok(())
}

// =============================================================================
// Formatting
// =============================================================================

fn format_regulation_list(regulations: &HashMap<String, RegulationInfo>) -> Vec<RegulationListItem> {
    let mut result: Vec<RegulationListItem> = regulations
        .iter()
        .map(|(id, info)| {
            let data = &info.data;
            RegulationListItem {
                id: id.clone(),
                name: data
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| id.replace('_', " ")),
                regulatory_layer: data
                    .get("regulatory_layer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ONBEKEND")
                    .to_string(),
                publication_date: data
                    .get("publication_date")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                valid_from: data
                    .get("valid_from")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                bwb_id: data
                    .get("bwb_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                url: data.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                article_count: data
                    .get("articles")
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0),
            }
        })
        .collect();

    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}

fn format_article(article: &Value, annotations: &[Value]) -> Value {
    let machine_readable = article.get("machine_readable").cloned().unwrap_or(json!({}));
    let execution = machine_readable.get("execution").cloned().unwrap_or(json!({}));

    let article_nr = article
        .get("number")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Filter annotations for this article
    let article_annotations: Vec<&Value> = annotations
        .iter()
        .filter(|ann| {
            ann.get("target")
                .and_then(|t| t.get("article"))
                .map(|a| match a {
                    Value::String(s) => s == article_nr,
                    Value::Number(n) => n.to_string() == article_nr,
                    _ => false,
                })
                .unwrap_or(false)
        })
        .collect();

    json!({
        "number": article.get("number"),
        "text": article.get("text"),
        "url": article.get("url"),
        "definitions": machine_readable.get("definitions").unwrap_or(&json!({})),
        "parameters": execution.get("parameters").unwrap_or(&json!([])),
        "input": execution.get("input").unwrap_or(&json!([])),
        "output": execution.get("output").unwrap_or(&json!([])),
        "actions": execution.get("actions").unwrap_or(&json!([])),
        "produces": execution.get("produces").unwrap_or(&json!({})),
        "machine_readable": {
            "open_norms": machine_readable.get("open_norms").unwrap_or(&json!([])),
            "requires_human_assessment": machine_readable.get("requires_human_assessment").unwrap_or(&json!(false)),
            "human_assessment_reason": machine_readable.get("human_assessment_reason"),
        },
        "annotations": article_annotations,
    })
}

// =============================================================================
// Handlers
// =============================================================================

async fn get_regulations(State(state): State<AppState>) -> impl IntoResponse {
    let regulations = state.regulations.read().unwrap();
    let list = format_regulation_list(&regulations);
    Json(list)
}

async fn get_regulation(
    State(state): State<AppState>,
    Path(reg_id): Path<String>,
) -> impl IntoResponse {
    if !validate_regulation_id(&reg_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid regulation ID format"})),
        )
            .into_response();
    }

    let regulations = state.regulations.read().unwrap();

    match regulations.get(&reg_id) {
        Some(info) => {
            let annotations = load_annotations(&state.annotations_dir, &reg_id);
            let data = &info.data;

            let articles: Vec<Value> = data
                .get("articles")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().map(|art| format_article(art, &annotations)).collect())
                .unwrap_or_default();

            let raw_yaml = serde_yaml::to_string(data).unwrap_or_default();

            let response = json!({
                "id": data.get("$id"),
                "name": data.get("name").and_then(|v| v.as_str()).unwrap_or(&reg_id.replace('_', " ")),
                "regulatory_layer": data.get("regulatory_layer"),
                "publication_date": data.get("publication_date"),
                "valid_from": data.get("valid_from"),
                "bwb_id": data.get("bwb_id"),
                "url": data.get("url"),
                "articles": articles,
                "relations": {},
                "raw_yaml": raw_yaml,
            });

            Json(response).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Regulation '{}' not found", reg_id)})),
        )
            .into_response(),
    }
}

async fn get_annotations(
    State(state): State<AppState>,
    Path(reg_id): Path<String>,
) -> impl IntoResponse {
    if !validate_regulation_id(&reg_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid regulation ID format"})),
        )
            .into_response();
    }

    let annotations = load_annotations(&state.annotations_dir, &reg_id);
    Json(json!({"annotations": annotations})).into_response()
}

async fn add_annotation(
    State(state): State<AppState>,
    Path(reg_id): Path<String>,
    Json(mut annotation): Json<Value>,
) -> impl IntoResponse {
    if !validate_regulation_id(&reg_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid regulation ID format"})),
        )
            .into_response();
    }

    // Parse and validate annotation
    let ann_request: AnnotationRequest = match serde_json::from_value(annotation.clone()) {
        Ok(ann) => ann,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": format!("Invalid annotation format: {}", e)})),
            )
                .into_response();
        }
    };

    if let Err(e) = validate_annotation(&ann_request) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Invalid annotation: {}", e)})),
        )
            .into_response();
    }

    // Load existing annotations
    let mut annotations = load_annotations(&state.annotations_dir, &reg_id);

    // Check for duplicates
    let target = &ann_request.target;
    let exact_text = target
        .selector
        .as_ref()
        .and_then(|s| s.exact.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");
    let article = target.article.as_ref().map(|s| s.as_str()).unwrap_or("");

    for existing in &annotations {
        let ex_target = existing.get("target");
        let ex_selector = ex_target.and_then(|t| t.get("selector"));
        let ex_exact = ex_selector
            .and_then(|s| s.get("exact"))
            .and_then(|e| e.as_str())
            .unwrap_or("");
        let ex_article = ex_target
            .and_then(|t| t.get("article"))
            .map(|a| match a {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => a.to_string(),
            })
            .unwrap_or_default();

        if ex_exact == exact_text && ex_article == article {
            return Json(json!({"status": "exists", "message": "Annotation already exists"}))
                .into_response();
        }
    }

    // Add default status if not present
    if annotation.get("status").is_none() {
        annotation["status"] = json!("draft");
    }

    annotations.push(annotation.clone());

    // Save annotations
    match save_annotations(&state.annotations_dir, &reg_id, &annotations) {
        Ok(_) => {
            info!("Saved W3C annotation to {}.yaml", reg_id);
            Json(json!({"status": "ok", "annotation": annotation})).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to save: {}", e)})),
        )
            .into_response(),
    }
}

async fn change_annotation_status(
    State(state): State<AppState>,
    Path((reg_id, ann_idx)): Path<(String, usize)>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    if !validate_regulation_id(&reg_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid regulation ID format"})),
        )
            .into_response();
    }

    let new_status = match body.get("status").and_then(|s| s.as_str()) {
        Some(s) if ["draft", "approved", "rejected", "promoted"].contains(&s) => s.to_string(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Invalid status (must be draft, approved, rejected, or promoted)"})),
            )
                .into_response();
        }
    };

    let mut annotations = load_annotations(&state.annotations_dir, &reg_id);

    if ann_idx >= annotations.len() {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Annotation index {} not found", ann_idx)})),
        )
            .into_response();
    }

    annotations[ann_idx]["status"] = json!(new_status.clone());

    match save_annotations(&state.annotations_dir, &reg_id, &annotations) {
        Ok(_) => Json(json!({
            "status": "ok",
            "annotation_status": new_status,
            "index": ann_idx
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to save: {}", e)})),
        )
            .into_response(),
    }
}

async fn get_relations(State(_state): State<AppState>) -> impl IntoResponse {
    // Return empty relations for now - can be extended later
    Json(json!({}))
}

async fn get_bwb_mappings(State(state): State<AppState>) -> impl IntoResponse {
    // Build BWB mappings from loaded regulations
    let regulations = state.regulations.read().unwrap();
    let mut mappings: HashMap<String, Value> = HashMap::new();

    for (law_id, info) in regulations.iter() {
        if let Some(bwb_id) = info.data.get("bwb_id").and_then(|v| v.as_str()) {
            let name = info
                .data
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(law_id);
            let url = info.data.get("url").and_then(|v| v.as_str()).unwrap_or("");

            mappings.insert(
                bwb_id.to_string(),
                json!({
                    "bwb_id": bwb_id,
                    "law_id": law_id,
                    "name": name,
                    "url": url
                }),
            );
        }
    }

    Json(json!(mappings))
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Determine paths (relative to project root, assuming we're running from test5/)
    let project_root = PathBuf::from("../../..");
    let regulation_dir = project_root.join("regulation/nl");
    let annotations_dir = project_root.join("annotations");
    let frontend_dir = project_root.join("frontend");

    // Load regulations
    info!("Loading regulations from {:?}...", regulation_dir);
    let regulations = load_all_regulations(&regulation_dir);
    info!("Loaded {} regulations", regulations.len());

    let state = AppState {
        regulations: Arc::new(RwLock::new(regulations)),
        regulation_dir,
        annotations_dir,
        frontend_dir: frontend_dir.clone(),
    };

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/regulations", get(get_regulations))
        .route("/api/regulation/:id", get(get_regulation))
        .route("/api/regulation/:id/annotations", get(get_annotations))
        .route("/api/regulation/:id/annotation", post(add_annotation))
        .route(
            "/api/annotation/:reg_id/:ann_idx/status",
            post(change_annotation_status),
        )
        .route("/api/bwb", get(get_bwb_mappings))
        .route("/api/relations", get(get_relations))
        // Static files
        .nest_service("/", ServeDir::new(&frontend_dir))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server (localhost only for security)
    let addr = "127.0.0.1:8000";
    info!("HighlightEditor server starting on http://{}", addr);
    info!("Serving frontend from {:?}", frontend_dir);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
