//! Internazionalizzazione minima: due dizionari piatti (IT/EN).
//!
//! Ogni stringa visibile passa da [`Locale::get`]. Se una chiave manca, si mostra la
//! chiave stessa (mai un crash): i buchi si trovano con
//! `cargo run -p xtask -- check-locales` e con il test `locales_have_same_keys`.

use std::collections::HashMap;

/// Lingua dell'interfaccia con il suo dizionario.
#[derive(Clone, Debug)]
pub struct Locale {
    /// Codice lingua (`it`, `en`).
    pub code: String,
    strings: HashMap<String, String>,
}

impl Locale {
    /// Italiano (lingua predefinita).
    pub fn italian() -> Self {
        Self::from_json("it", include_str!("../../../locales/it.json"))
    }

    /// Inglese.
    pub fn english() -> Self {
        Self::from_json("en", include_str!("../../../locales/en.json"))
    }

    fn from_json(code: &str, json: &str) -> Self {
        let strings: HashMap<String, String> =
            serde_json::from_str(json).expect("locales/*.json non valido");
        Self {
            code: code.to_string(),
            strings,
        }
    }

    /// Restituisce la stringa per la chiave, o la chiave stessa se manca.
    ///
    /// La chiave è `&'static str`: tutte le stringhe dell'interfaccia sono letterali
    /// (vedi `CONTRIBUTING.md`), così non serve allocare a ogni frame.
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
            "chiavi non allineate — solo IT: {only_it:?}, solo EN: {only_en:?}"
        );
        assert!(!it.is_empty());
    }

    #[test]
    fn missing_key_returns_key() {
        let locale = Locale::italian();
        assert_eq!(locale.get("chiave_che_non_esiste"), "chiave_che_non_esiste");
        assert_eq!(locale.get("menu_file"), "File");
    }
}
