//! Regulation loader for BDD tests
//!
//! Loads all YAML regulation files from the regulation/nl directory.

use regelrecht_engine::{EngineError, LawExecutionService};
use std::path::Path;
use walkdir::WalkDir;

/// Load all regulation YAML files into the service.
///
/// Scans the `regulation/nl/` directory relative to the package manifest
/// and loads all `.yaml` files found.
pub fn load_all_regulations(service: &mut LawExecutionService) -> Result<usize, EngineError> {
    // Find the regulation directory relative to Cargo.toml
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let regulation_dir = Path::new(manifest_dir)
        .parent() // packages/
        .and_then(|p| p.parent()) // project root
        .map(|p| p.join("regulation").join("nl"))
        .ok_or_else(|| EngineError::LoadError("Could not find regulation directory".to_string()))?;

    if !regulation_dir.exists() {
        return Err(EngineError::LoadError(format!(
            "Regulation directory not found: {}",
            regulation_dir.display()
        )));
    }

    let mut count = 0;

    for entry in WalkDir::new(&regulation_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only process YAML files
        if path.is_file() && path.extension().is_some_and(|ext| ext == "yaml") {
            let content = std::fs::read_to_string(path).map_err(|e| {
                EngineError::LoadError(format!("Failed to read {}: {}", path.display(), e))
            })?;

            match service.load_law(&content) {
                Ok(law_id) => {
                    tracing::debug!(law_id = %law_id, path = %path.display(), "Loaded law");
                    count += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to load law file (skipping)"
                    );
                    // Continue loading other files even if one fails
                }
            }
        }
    }

    tracing::info!(count = count, "Loaded regulations");
    Ok(count)
}

/// Get the path to a specific regulation file.
#[allow(dead_code)]
pub fn get_regulation_path(relative_path: &str) -> Option<std::path::PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir)
        .parent()?
        .parent()?
        .join("regulation")
        .join("nl")
        .join(relative_path);

    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::load_all_regulations;
    use regelrecht_engine::LawExecutionService;

    #[test]
    fn test_load_all_regulations() {
        let mut service = LawExecutionService::new();
        let count = load_all_regulations(&mut service).expect("Failed to load regulations");
        assert!(count > 0, "Expected to load at least one regulation");
    }

    #[test]
    fn test_specific_laws_loaded() {
        let mut service = LawExecutionService::new();
        load_all_regulations(&mut service).expect("Failed to load regulations");

        // Check that key laws are loaded
        assert!(
            service.has_law("participatiewet"),
            "participatiewet should be loaded"
        );
        assert!(
            service.has_law("burgerlijk_wetboek_boek_5"),
            "BW5 should be loaded"
        );
    }
}
