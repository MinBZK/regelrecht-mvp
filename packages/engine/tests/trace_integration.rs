//! Integration tests for execution tracing.
//!
//! Verifies that the trace output matches the expected box-drawing format
//! for the zorgtoeslag (healthcare allowance) scenario.

use regelrecht_engine::{LawExecutionService, Value};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Load all regulation YAML files into the service.
fn load_all_regulations(service: &mut LawExecutionService) -> Result<usize, String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let regulation_dir = Path::new(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("regulation").join("nl"))
        .ok_or_else(|| "Could not find regulation directory".to_string())?;

    if !regulation_dir.exists() {
        return Err(format!(
            "Regulation directory not found: {}",
            regulation_dir.display()
        ));
    }

    let mut count = 0;
    for entry in WalkDir::new(&regulation_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            if service.load_law(&content).is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Set up a service with all regulations loaded and zorgtoeslag data registered.
fn setup_zorgtoeslag_service() -> LawExecutionService {
    let mut service = LawExecutionService::new();
    load_all_regulations(&mut service).expect("Failed to load regulations");

    // Register derived data for zorgtoeslag Article 2 inputs.
    // This mirrors the BDD step `execute_healthcare_allowance`.
    let mut derived_record: HashMap<String, Value> = HashMap::new();
    derived_record.insert(
        "bsn".to_string(),
        Value::String("999993653".to_string()),
    );
    derived_record.insert("leeftijd".to_string(), Value::Int(20));
    derived_record.insert("is_verzekerde".to_string(), Value::Bool(true));
    derived_record.insert("heeft_toeslagpartner".to_string(), Value::Bool(false));
    derived_record.insert("toetsingsinkomen".to_string(), Value::Int(79547));

    service
        .register_dict_source("derived_zorgtoeslag_inputs", "bsn", vec![derived_record])
        .expect("Failed to register derived data source");

    service
}

#[test]
fn test_zorgtoeslag_trace_output_format() {
    let service = setup_zorgtoeslag_service();

    let mut params = HashMap::new();
    params.insert(
        "bsn".to_string(),
        Value::String("999993653".to_string()),
    );
    params.insert("vermogen".to_string(), Value::Int(0));
    params.insert("heeft_toeslagpartner".to_string(), Value::Bool(false));

    let result = service
        .evaluate_law_output_with_trace(
            "zorgtoeslagwet",
            "hoogte_zorgtoeslag",
            params,
            "2025-01-01",
        )
        .expect("Law evaluation should succeed");

    // Verify the computation result
    let hoogte = result.outputs.get("hoogte_zorgtoeslag");
    assert!(
        hoogte.is_some(),
        "Expected hoogte_zorgtoeslag output, got: {:?}",
        result.outputs.keys().collect::<Vec<_>>()
    );

    // The trace should be populated
    let trace = result.trace.expect("Trace should be populated");

    // Render the box-drawing trace
    let rendered = trace.render_box_drawing();

    // Snapshot comparison against expected trace output
    let expected = include_str!("expected_zorgtoeslag_trace.txt");
    assert_eq!(
        rendered.trim(),
        expected.trim(),
        "Trace output does not match expected snapshot.\n\n--- ACTUAL ---\n{}\n--- EXPECTED ---\n{}",
        rendered,
        expected
    );
}

#[test]
fn test_zorgtoeslag_trace_result_matches_non_trace() {
    let service = setup_zorgtoeslag_service();

    let mut params = HashMap::new();
    params.insert(
        "bsn".to_string(),
        Value::String("999993653".to_string()),
    );
    params.insert("vermogen".to_string(), Value::Int(0));
    params.insert("heeft_toeslagpartner".to_string(), Value::Bool(false));

    // Execute with trace
    let traced_result = service
        .evaluate_law_output_with_trace(
            "zorgtoeslagwet",
            "hoogte_zorgtoeslag",
            params.clone(),
            "2025-01-01",
        )
        .expect("Traced evaluation should succeed");

    // Execute without trace
    let normal_result = service
        .evaluate_law_output(
            "zorgtoeslagwet",
            "hoogte_zorgtoeslag",
            params,
            "2025-01-01",
        )
        .expect("Normal evaluation should succeed");

    // Results should be identical
    assert_eq!(
        traced_result.outputs, normal_result.outputs,
        "Traced and non-traced results should be identical"
    );
}

#[test]
fn test_trace_disabled_by_default() {
    let service = setup_zorgtoeslag_service();

    let mut params = HashMap::new();
    params.insert(
        "bsn".to_string(),
        Value::String("999993653".to_string()),
    );
    params.insert("vermogen".to_string(), Value::Int(0));
    params.insert("heeft_toeslagpartner".to_string(), Value::Bool(false));

    // Normal evaluation should not have a trace
    let result = service
        .evaluate_law_output(
            "zorgtoeslagwet",
            "hoogte_zorgtoeslag",
            params,
            "2025-01-01",
        )
        .expect("Evaluation should succeed");

    assert!(
        result.trace.is_none(),
        "Normal evaluation should not produce a trace"
    );
}

#[test]
fn test_simple_law_trace() {
    // Test tracing with a simple single-law scenario (standard premium)
    let mut service = LawExecutionService::new();
    load_all_regulations(&mut service).expect("Failed to load regulations");

    let result = service
        .evaluate_law_output_with_trace(
            "regeling_standaardpremie",
            "standaardpremie",
            HashMap::new(),
            "2025-01-01",
        )
        .expect("Standard premium evaluation should succeed");

    let trace = result.trace.expect("Trace should be populated");
    let rendered = trace.render_box_drawing();

    // Snapshot comparison against expected trace output
    let expected = include_str!("expected_standaardpremie_trace.txt");
    assert_eq!(
        rendered.trim(),
        expected.trim(),
        "Trace output does not match expected snapshot.\n\n--- ACTUAL ---\n{}\n--- EXPECTED ---\n{}",
        rendered,
        expected
    );
}
