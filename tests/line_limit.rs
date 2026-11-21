//! Repository guard: no source file grows past 500 lines.
//!
//! Build output (`target/`) and `node_modules/` are excluded; every tracked Rust source
//! under `src`, `tests`, `examples`, `templates`, `xtask`, and `web/src` is checked.
//! Keep the limit at 500 lines and split by component, not by line count.

use std::fs;
use std::path::{Path, PathBuf};

const LIMIT: usize = 500;

#[test]
fn source_files_stay_within_500_lines() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests", "examples", "templates", "xtask", "web/src"] {
        collect(&root.join(directory), &mut files);
    }

    let mut offenders = Vec::new();
    for path in &files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let lines = text.lines().count();
        if lines > LIMIT {
            offenders.push(format!("{}: {lines} lines", relative(root, path)));
        }
    }

    assert!(
        offenders.is_empty(),
        "source files over {LIMIT} lines:\n{}",
        offenders.join("\n")
    );
    assert!(!files.is_empty(), "no Rust sources found under {root:?}");
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if name != "target" && name != "node_modules" && name != ".git" {
                collect(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}
