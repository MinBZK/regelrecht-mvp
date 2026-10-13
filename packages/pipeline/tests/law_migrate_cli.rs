//! What the exit code and report of `law-migrate` say.
//!
//! A file that migrates clean leaves the run green; a file that cannot be
//! migrated turns it red. A file already on a newer schema than the target
//! passes through, and the report says nothing changed.

// Allowed crate-wide: test helpers outside a `#[test]` fn may unwrap, expect and
// panic too, because that is how a failing fixture reports itself.
// `allow-*-in-tests` in clippy.toml only reaches `#[test]` fns.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::process::Command;

/// A schema-valid law on v0.8.0, newer than the migration target.
const LAW_ON_V0_8_0: &str = r#"$schema: https://raw.githubusercontent.com/MinBZK/regelrecht/refs/tags/schema-v0.8.0/schema/v0.8.0/schema.json
$id: testwet
regulatory_layer: WET
publication_date: '2025-12-20'
valid_from: '2026-01-01'
bwb_id: BWBR0000001
url: https://example.com/BWBR0000001
name: Testwet
articles:
  - number: '1'
    text: Een artikel zonder model.
    url: https://example.com/BWBR0000001#Artikel1
"#;

fn run(file: &Path) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_law-migrate"))
        .arg(file)
        .output()
        .expect("the binary is built by cargo test");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

fn law_file(dir: &Path, yaml: &str) -> std::path::PathBuf {
    let path = dir.join("2026-01-01.yaml");
    std::fs::write(&path, yaml).unwrap();
    path
}

#[test]
fn a_file_on_a_newer_schema_passes_through_and_the_run_is_green() {
    let dir = tempfile::tempdir().unwrap();
    let (code, out) = run(&law_file(dir.path(), LAW_ON_V0_8_0));
    assert_eq!(code, Some(0), "{out}");
    assert!(out.contains("change:    none"), "{out}");
}

#[test]
fn a_file_that_cannot_be_migrated_turns_the_run_red() {
    let dir = tempfile::tempdir().unwrap();
    let (code, out) = run(&law_file(dir.path(), "dit is: [geen geldige yaml"));
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("cannot migrate"), "{out}");
}
