//! CLI binary for evaluating a law YAML via stdin.
//!
//! Usage:
//!   echo '{"law_yaml": "...", "output_name": "...", "params": {...}, "date": "2025-01-01"}' \
//!     | cargo run --bin evaluate
//!
//! Input (JSON on stdin):
//!   - law_yaml: String — the full YAML content of the law
//!   - output_name: String — the output to evaluate (e.g. "heeft_recht_op_zorgtoeslag")
//!   - params: Object — key-value parameters to pass to the engine
//!   - date: String — evaluation date (YYYY-MM-DD)
//!   - extra_laws: Optional<Vec<String>> — additional YAML laws for cross-law resolution
//!
//! Output (JSON on stdout):
//!   - outputs: Object — computed output values
//!   - resolved_inputs: Object — resolved input values from cross-law references
//!   - error: Optional<String> — error message if evaluation failed

use regelrecht_engine::{LawExecutionService, Value};
use std::collections::HashMap;
use std::io::Read;

#[derive(serde::Deserialize)]
struct EvaluateRequest {
    law_yaml: String,
    output_name: String,
    params: HashMap<String, serde_json::Value>,
    date: String,
    #[serde(default)]
    extra_laws: Vec<String>,
}

#[derive(serde::Serialize)]
struct EvaluateResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    outputs: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resolved_inputs: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::Array(arr.iter().map(json_to_value).collect()),
        _ => Value::Null,
    }
}

fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::json!(*i),
        Value::Float(f) => serde_json::json!(*f),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_json).collect()),
        Value::Object(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}

fn main() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        let resp = EvaluateResponse {
            outputs: None,
            resolved_inputs: None,
            error: Some(format!("Failed to read stdin: {e}")),
        };
        println!("{}", serde_json::to_string(&resp).unwrap_or_default());
        std::process::exit(1);
    }

    let request: EvaluateRequest = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(e) => {
            let resp = EvaluateResponse {
                outputs: None,
                resolved_inputs: None,
                error: Some(format!("Failed to parse request JSON: {e}")),
            };
            println!("{}", serde_json::to_string(&resp).unwrap_or_default());
            std::process::exit(1);
        }
    };

    let mut service = LawExecutionService::new();

    // Load the primary law
    if let Err(e) = service.load_law(&request.law_yaml) {
        let resp = EvaluateResponse {
            outputs: None,
            resolved_inputs: None,
            error: Some(format!("Failed to load law YAML: {e}")),
        };
        println!("{}", serde_json::to_string(&resp).unwrap_or_default());
        std::process::exit(1);
    }

    // Load additional laws for cross-law resolution
    for extra_yaml in &request.extra_laws {
        if let Err(e) = service.load_law(extra_yaml) {
            let resp = EvaluateResponse {
                outputs: None,
                resolved_inputs: None,
                error: Some(format!("Failed to load extra law YAML: {e}")),
            };
            println!("{}", serde_json::to_string(&resp).unwrap_or_default());
            std::process::exit(1);
        }
    }

    // Convert params
    let params: HashMap<String, Value> = request
        .params
        .iter()
        .map(|(k, v)| (k.clone(), json_to_value(v)))
        .collect();

    // Get the law ID from the first loaded law
    let law_ids = service.list_laws();
    let law_id: String = if let Some(id) = law_ids.first() {
        id.to_string()
    } else {
        let resp = EvaluateResponse {
            outputs: None,
            resolved_inputs: None,
            error: Some("No laws loaded".to_string()),
        };
        println!("{}", serde_json::to_string(&resp).unwrap_or_default());
        std::process::exit(1);
    };

    // Evaluate
    match service.evaluate_law_output(&law_id, &request.output_name, params, &request.date) {
        Ok(result) => {
            let outputs: HashMap<String, serde_json::Value> = result
                .outputs
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            let resolved_inputs: HashMap<String, serde_json::Value> = result
                .resolved_inputs
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            let resp = EvaluateResponse {
                outputs: Some(outputs),
                resolved_inputs: Some(resolved_inputs),
                error: None,
            };
            println!("{}", serde_json::to_string(&resp).unwrap_or_default());
        }
        Err(e) => {
            let resp = EvaluateResponse {
                outputs: None,
                resolved_inputs: None,
                error: Some(format!("{e}")),
            };
            println!("{}", serde_json::to_string(&resp).unwrap_or_default());
            std::process::exit(1);
        }
    }
}
