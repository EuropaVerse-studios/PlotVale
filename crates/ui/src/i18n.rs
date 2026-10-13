//! Minimal internationalization: two flat dictionaries (EN/IT).
//!
//! Every visible string goes through [`Locale::get`]. A missing key renders as
//! the key itself (never a crash): gaps are caught by
//! `cargo run -p xtask -- check-locales` and the `locales_have_same_keys` test.

use std::collections::HashMap;

/// Interface language with its dictionary.
#[derive(Clone, Debug)]
pub struct Locale {
    /// Language code (`it`, `en`).
    pub code: String,
    strings: HashMap<String, String>,
}

impl Locale {
    /// Italian (default language).
    pub fn italian() -> Self {
        Self::from_json("it", include_str!("../../../locales/it.json"))
    }

    /// English.
    pub fn english() -> Self {
        Self::from_json("en", include_str!("../../../locales/en.json"))
    }

    fn from_json(code: &str, json: &str) -> Self {
        let strings: HashMap<String, String> =
            serde_json::from_str(json).expect("invalid locales/*.json");
        Self {
            code: code.to_string(),
            strings,
        }
    }

    /// Returns the string for the key, or the key itself if missing.
    ///
    /// The key is `&'static str`: all interface strings are literals
    /// (see `CONTRIBUTING.md`), so no per-frame allocation is needed.
    pub fn get(&self, key: &'static str) -> &str {
        self.strings.get(key).map(String::as_str).unwrap_or(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(json: &str) -> std::collections::BTreeSet<String> {
        let map: HashMap<String, String> = serde_json::from_str(json).unwrap();
        map.into_keys().collect()
    }

    #[test]
    fn locales_have_same_keys() {
        let it = keys(include_str!("../../../locales/it.json"));
        let en = keys(include_str!("../../../locales/en.json"));
        let only_it: Vec<_> = it.difference(&en).collect();
        let only_en: Vec<_> = en.difference(&it).collect();
        assert!(
            only_it.is_empty() && only_en.is_empty(),
            "misaligned keys — only IT: {only_it:?}, only EN: {only_en:?}"
        );
        assert!(!it.is_empty());
    }

    #[test]
    fn missing_key_returns_key() {
        let locale = Locale::italian();
        assert_eq!(locale.get("nonexistent_key"), "nonexistent_key");
        assert_eq!(locale.get("menu_file"), "File");
    }
}
