//! Helpers the cell's tests share.

/// An empty directory for a cell, with only its identity (`cel.yaml`,
/// RFC-046).
pub fn cell_tempdir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("cel.yaml"), "name: Testcel\n").unwrap();
    dir
}
