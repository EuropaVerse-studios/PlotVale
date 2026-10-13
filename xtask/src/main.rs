//! Automazioni di PlotVale (`cargo run -p xtask -- <task>`).
//!
//! Task disponibili:
//! - `check-locales`: verifica che `locales/it.json` e `locales/en.json`
//!   contengano esattamente le stesse chiavi.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn main() {
    let task = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "help".to_string());
    let code = match task.as_str() {
        "check-locales" => check_locales(),
        _ => {
            println!("Uso: cargo run -p xtask -- <task>");
            println!("Task: check-locales");
            0
        }
    };
    std::process::exit(code);
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask dentro il workspace")
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
                eprintln!("xtask: impossibile leggere {}: {e}", path.display());
                return 1;
            }
        };
        let map: std::collections::HashMap<String, String> = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("xtask: {} non è JSON valido: {e}", path.display());
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
            eprintln!("xtask: chiave '{missing}' presente in {first_lang} ma non in {lang}");
            ok = false;
        }
        for extra in keys.difference(&first_keys) {
            eprintln!("xtask: chiave '{extra}' presente in {lang} ma non in {first_lang}");
            ok = false;
        }
    }
    if ok {
        println!("xtask: locales OK ({} chiavi)", first_keys.len());
        0
    } else {
        1
    }
}
