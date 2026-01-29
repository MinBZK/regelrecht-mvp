use std::path::Path;
use std::process;

use regelrecht_engine::article::ArticleBasedLaw;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: validate <file1.yaml> [file2.yaml ...]");
        process::exit(1);
    }

    let mut failed = false;

    for arg in &args {
        let path = Path::new(arg);
        match ArticleBasedLaw::from_yaml_file(path) {
            Ok(_) => {
                eprintln!("OK: {}", path.display());
            }
            Err(e) => {
                eprintln!("FAIL: {}: {e}", path.display());
                failed = true;
            }
        }
    }

    if failed {
        process::exit(1);
    }
}
