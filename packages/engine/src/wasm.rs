//! WASM bindings for the RegelRecht engine
//!
//! This module provides WebAssembly bindings that match the Python `LawExecutionService` API.
//! It is feature-gated behind the `wasm` feature flag.
//!
//! # Key Constraints
//!
//! - **No filesystem access in WASM**: Laws must be passed as YAML strings via `load_law()`
//! - **Efficient serialization**: Uses `serde-wasm-bindgen` for Rust <-> JavaScript conversion
//!
//! # Limitations
//!
//! - **No cross-law resolution**: External references (`source.regulation`) require a
//!   ServiceProvider implementation (not yet available in WASM). Workaround: pre-resolve
//!   external values and pass them as parameters.
//! - **No delegation resolution**: Delegation with `select_on` criteria requires ServiceProvider.
//!   Workaround: pre-resolve delegated values and pass them as parameters.
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { WasmEngine } from 'regelrecht-engine';
//!
//! await init();
//! const engine = new WasmEngine();
//!
//! // Load law from HTTP
//! const response = await fetch('/laws/zorgtoeslagwet.yaml');
//! const yaml = await response.text();
//! const lawId = engine.loadLaw(yaml);
//!
//! // Execute (calculation_date is required)
//! const result = engine.execute(
//!     'zorgtoeslagwet',
//!     'heeft_recht_op_zorgtoeslag',
//!     { BSN: '123456789', vermogen: 50000, heeft_toeslagpartner: false },
//!     '2025-01-01'
//! );
//! console.log(result.outputs);
//! ```

use serde::Serialize;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::article::ArticleBasedLaw;
use crate::engine::ArticleEngine;
use crate::error::EngineError;
use crate::types::{RegulatoryLayer, Value};

/// Maximum YAML size to prevent DoS attacks (1 MB)
const MAX_YAML_SIZE: usize = 1_000_000;

/// Maximum number of laws that can be loaded
const MAX_LOADED_LAWS: usize = 100;

/// Helper to create consistent error JsValues
fn wasm_error(msg: &str) -> JsValue {
    JsValue::from_str(msg)
}

/// Serializable result for execute() - avoids double serialization through serde_json
#[derive(Serialize)]
struct WasmExecuteResult {
    outputs: HashMap<String, Value>,
    resolved_inputs: HashMap<String, Value>,
    article_number: String,
    law_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    law_uuid: Option<String>,
}

/// Serializable law info for get_law_info()
#[derive(Serialize)]
struct WasmLawInfo {
    id: String,
    regulatory_layer: RegulatoryLayer,
    publication_date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bwb_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    outputs: Vec<String>,
    article_count: usize,
}

/// WASM-compatible law execution engine.
///
/// Provides the same functionality as Python's `LawExecutionService`, but adapted
/// for WASM constraints (no filesystem access).
#[wasm_bindgen]
pub struct WasmEngine {
    laws: HashMap<String, ArticleBasedLaw>,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Create a new empty engine instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            laws: HashMap::new(),
        }
    }

    /// Load a law from a YAML string.
    ///
    /// # Arguments
    /// * `yaml` - YAML string containing the law definition (max 1 MB)
    ///
    /// # Returns
    /// * `Ok(String)` - The law ID (used for subsequent `execute()` calls)
    /// * `Err(JsValue)` - Error message if parsing fails
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const response = await fetch('/laws/zorgtoeslagwet.yaml');
    /// const yaml = await response.text();
    /// const lawId = engine.loadLaw(yaml);  // Returns "zorgtoeslagwet"
    /// ```
    #[wasm_bindgen(js_name = loadLaw)]
    pub fn load_law(&mut self, yaml: &str) -> Result<String, JsValue> {
        // Input validation
        if yaml.len() > MAX_YAML_SIZE {
            return Err(wasm_error("YAML exceeds maximum size (1 MB)"));
        }
        if self.laws.len() >= MAX_LOADED_LAWS {
            return Err(wasm_error("Maximum number of laws reached (100)"));
        }

        let law = ArticleBasedLaw::from_yaml_str(yaml).map_err(EngineError::from)?;
        let id = law.id.clone();
        self.laws.insert(id.clone(), law);
        Ok(id)
    }

    /// Execute an article's output with the given parameters.
    ///
    /// # Arguments
    /// * `law_id` - ID of the loaded law (returned by `loadLaw()`)
    /// * `output_name` - Name of the output to calculate
    /// * `parameters` - JavaScript object with input parameters
    /// * `calculation_date` - Date string (YYYY-MM-DD) for which to calculate
    ///
    /// # Returns
    /// * `Ok(JsValue)` - JavaScript object with `outputs`, `resolved_inputs`, etc.
    /// * `Err(JsValue)` - Error message if execution fails
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const result = engine.execute(
    ///     'zorgtoeslagwet',
    ///     'heeft_recht_op_zorgtoeslag',
    ///     { vermogen: 50000, heeft_toeslagpartner: false },
    ///     '2025-01-01'
    /// );
    /// console.log(result.outputs.heeft_recht_op_zorgtoeslag);  // true or false
    /// ```
    pub fn execute(
        &self,
        law_id: &str,
        output_name: &str,
        parameters: JsValue,
        calculation_date: &str,
    ) -> Result<JsValue, JsValue> {
        // Find law
        let law = self
            .laws
            .get(law_id)
            .ok_or_else(|| EngineError::LawNotFound(law_id.to_string()))?;

        // Find article by output
        let article = law.find_article_by_output(output_name).ok_or_else(|| {
            EngineError::OutputNotFound {
                law_id: law_id.to_string(),
                output: output_name.to_string(),
            }
        })?;

        // Parse parameters from JsValue
        let params: HashMap<String, Value> = serde_wasm_bindgen::from_value(parameters)
            .map_err(|e| wasm_error(&format!("Failed to parse parameters: {}", e)))?;

        // Execute
        let engine = ArticleEngine::new(article, law);
        let result = engine.evaluate(params, calculation_date)?;

        // Serialize result directly (no intermediate serde_json::Value)
        let wasm_result = WasmExecuteResult {
            outputs: result.outputs,
            resolved_inputs: result.resolved_inputs,
            article_number: result.article_number,
            law_id: result.law_id,
            law_uuid: result.law_uuid,
        };

        serde_wasm_bindgen::to_value(&wasm_result)
            .map_err(|e| wasm_error(&format!("Failed to serialize result: {}", e)))
    }

    /// List all loaded law IDs (sorted alphabetically).
    ///
    /// # Returns
    /// Array of law ID strings in alphabetical order.
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const laws = engine.listLaws();  // ["awir", "zorgtoeslagwet", ...]
    /// ```
    #[wasm_bindgen(js_name = listLaws)]
    pub fn list_laws(&self) -> Vec<String> {
        let mut keys: Vec<String> = self.laws.keys().cloned().collect();
        keys.sort();
        keys
    }

    /// Get metadata about a loaded law.
    ///
    /// # Arguments
    /// * `law_id` - ID of the law to query
    ///
    /// # Returns
    /// * `Ok(JsValue)` - JavaScript object with law metadata
    /// * `Err(JsValue)` - Error if law not found
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const info = engine.getLawInfo('zorgtoeslagwet');
    /// console.log(info.outputs);  // ["heeft_recht_op_zorgtoeslag", "hoogte_zorgtoeslag", ...]
    /// console.log(info.article_count);  // 5
    /// ```
    #[wasm_bindgen(js_name = getLawInfo)]
    pub fn get_law_info(&self, law_id: &str) -> Result<JsValue, JsValue> {
        let law = self
            .laws
            .get(law_id)
            .ok_or_else(|| EngineError::LawNotFound(law_id.to_string()))?;

        // Collect and sort outputs for consistent ordering
        let mut outputs: Vec<String> = law.get_all_outputs().keys().cloned().collect();
        outputs.sort();

        let info = WasmLawInfo {
            id: law.id.clone(),
            regulatory_layer: law.regulatory_layer.clone(),
            publication_date: law.publication_date.clone(),
            bwb_id: law.bwb_id.clone(),
            url: law.url.clone(),
            outputs,
            article_count: law.articles.len(),
        };

        serde_wasm_bindgen::to_value(&info)
            .map_err(|e| wasm_error(&format!("Failed to serialize law info: {}", e)))
    }

    /// Remove a loaded law from the engine.
    ///
    /// # Arguments
    /// * `law_id` - ID of the law to remove
    ///
    /// # Returns
    /// * `true` if the law was removed, `false` if it wasn't loaded
    #[wasm_bindgen(js_name = unloadLaw)]
    pub fn unload_law(&mut self, law_id: &str) -> bool {
        self.laws.remove(law_id).is_some()
    }

    /// Check if a law is loaded.
    ///
    /// # Arguments
    /// * `law_id` - ID of the law to check
    ///
    /// # Returns
    /// * `true` if the law is loaded, `false` otherwise
    #[wasm_bindgen(js_name = hasLaw)]
    pub fn has_law(&self, law_id: &str) -> bool {
        self.laws.contains_key(law_id)
    }

    /// Get the number of loaded laws.
    #[wasm_bindgen(js_name = lawCount)]
    pub fn law_count(&self) -> usize {
        self.laws.len()
    }

    /// Get the engine version.
    ///
    /// # Returns
    /// Version string (e.g., "0.1.0")
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Tests for WasmEngine
//
// Note: Most WASM-specific functionality (JsValue conversion, execute, get_law_info)
// can only be tested in an actual WASM environment. These tests focus on the
// non-WASM-dependent parts of the API.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::ArticleBasedLaw;

    const MINIMAL_LAW_YAML: &str = r#"
$id: test_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Test article
    machine_readable:
      execution:
        parameters:
          - name: value
            type: number
            required: true
        output:
          - name: result
            type: number
        actions:
          - output: result
            operation: MULTIPLY
            values:
              - $value
              - 2
"#;

    #[test]
    fn test_wasm_engine_new() {
        let engine = WasmEngine::new();
        assert_eq!(engine.law_count(), 0);
        assert!(engine.list_laws().is_empty());
    }

    #[test]
    fn test_wasm_engine_default() {
        let engine = WasmEngine::default();
        assert_eq!(engine.law_count(), 0);
    }

    #[test]
    fn test_wasm_engine_load_law_directly() {
        // Test the underlying law loading without going through JsValue conversion
        let mut engine = WasmEngine::new();

        // Manually parse and add the law (simulating what load_law does internally)
        let law = ArticleBasedLaw::from_yaml_str(MINIMAL_LAW_YAML).unwrap();
        let id = law.id.clone();
        engine.laws.insert(id.clone(), law);

        assert_eq!(engine.law_count(), 1);
        assert!(engine.has_law("test_law"));
        assert_eq!(engine.list_laws(), vec!["test_law".to_string()]);
    }

    #[test]
    fn test_wasm_engine_unload_law() {
        let mut engine = WasmEngine::new();

        // Add law directly
        let law = ArticleBasedLaw::from_yaml_str(MINIMAL_LAW_YAML).unwrap();
        engine.laws.insert(law.id.clone(), law);

        assert!(engine.has_law("test_law"));
        assert!(engine.unload_law("test_law"));
        assert!(!engine.has_law("test_law"));
        assert!(!engine.unload_law("nonexistent"));
    }

    #[test]
    fn test_wasm_engine_list_laws() {
        let mut engine = WasmEngine::new();

        // Add law directly
        let law = ArticleBasedLaw::from_yaml_str(MINIMAL_LAW_YAML).unwrap();
        engine.laws.insert(law.id.clone(), law);

        let laws = engine.list_laws();
        assert_eq!(laws.len(), 1);
        assert!(laws.contains(&"test_law".to_string()));
    }

    #[test]
    fn test_wasm_engine_has_law() {
        let mut engine = WasmEngine::new();
        assert!(!engine.has_law("test_law"));

        let law = ArticleBasedLaw::from_yaml_str(MINIMAL_LAW_YAML).unwrap();
        engine.laws.insert(law.id.clone(), law);

        assert!(engine.has_law("test_law"));
        assert!(!engine.has_law("other_law"));
    }

    #[test]
    fn test_wasm_engine_law_count() {
        let mut engine = WasmEngine::new();
        assert_eq!(engine.law_count(), 0);

        let law = ArticleBasedLaw::from_yaml_str(MINIMAL_LAW_YAML).unwrap();
        engine.laws.insert(law.id.clone(), law);
        assert_eq!(engine.law_count(), 1);

        let yaml2 = r#"
$id: second_law
regulatory_layer: WET
publication_date: '2025-01-01'
articles:
  - number: '1'
    text: Second test article
"#;
        let law2 = ArticleBasedLaw::from_yaml_str(yaml2).unwrap();
        engine.laws.insert(law2.id.clone(), law2);
        assert_eq!(engine.law_count(), 2);
    }
}
