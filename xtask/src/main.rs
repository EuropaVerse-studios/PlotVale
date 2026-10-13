//! PlotVale automation tasks (`cargo run -p xtask -- <task>`).
//!
//! Available tasks:
//! - `check-locales`: verifies that `locales/it.json` and `locales/en.json`
//!   contain exactly the same keys.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn main() {
    let task = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "help".to_string());
    let code = match task.as_str() {
        "check-locales" => check_locales(),
        _ => {
            println!("Usage: cargo run -p xtask -- <task>");
            println!("Tasks: check-locales");
            0
        }
    };
    std::process::exit(code);
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask inside the workspace")
        .to_path_buf()
}

fn check_locales() -> i32 {
    let root = workspace_root();
    let mut ok = true;
    let mut maps = Vec::new();
    for lang in ["it", "en"] {
        let path = root.join("locales").join(format!("{lang}.json"));
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("xtask: cannot read {}: {e}", path.display());
                return 1;
            }
        };
        let map: std::collections::HashMap<String, String> = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("xtask: {} is not valid JSON: {e}", path.display());
                return 1;
            }
        };
        maps.push((lang, map));
    }
    let (first_lang, first) = &maps[0];
    let first_keys: BTreeSet<&String> = first.keys().collect();
    for (lang, map) in &maps[1..] {
        let keys: BTreeSet<&String> = map.keys().collect();
        for missing in first_keys.difference(&keys) {
            eprintln!("xtask: key '{missing}' present in {first_lang} but not in {lang}");
            ok = false;
        }
        for extra in keys.difference(&first_keys) {
            eprintln!("xtask: key '{extra}' present in {lang} but not in {first_lang}");
            ok = false;
        }
    }
    if ok {
        println!("xtask: locales OK ({} keys)", first_keys.len());
        0
    } else {
        1
    }
}
