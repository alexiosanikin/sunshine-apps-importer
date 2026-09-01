//! Internationalization (i18n) support.
//!
//! Provides thread-safe translation lookup with fallback to English.
//! Uses `once_cell::sync::Lazy` for safe singleton initialization.

mod translations;

use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;

/// Supported language codes.
pub const LANG_EN: &str = "en";
pub const LANG_RU: &str = "ru";
pub const LANG_FR: &str = "fr";
pub const LANG_IT: &str = "it";
pub const LANG_ES: &str = "es";
pub const LANG_EL: &str = "el";
pub const LANG_KA: &str = "ka";

/// All valid language codes.
const VALID_LANGS: &[&str] = &[
    LANG_EN, LANG_RU, LANG_FR, LANG_IT, LANG_ES, LANG_EL, LANG_KA,
];

/// Thread-safe internationalization manager.
///
/// Uses a singleton pattern with lazy initialization. Translations are
/// loaded from JSON files in `src/i18n/translations/`.
pub struct I18n {
    translations: RwLock<HashMap<String, String>>,
    current_lang: RwLock<String>,
}

impl I18n {
    /// Look up a translation by key.
    ///
    /// Returns the key itself if no translation is found (graceful fallback).
    #[must_use]
    pub fn t(&self, key: &str) -> String {
        let translations = self.translations.read().unwrap();
        translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Get the current language code.
    #[must_use]
    pub fn current_lang(&self) -> String {
        self.current_lang.read().unwrap().clone()
    }

    /// Set the current language and load translations.
    ///
    /// Falls back to English if the language is invalid or loading fails.
    pub fn set_language(&self, lang: &str) -> Result<(), anyhow::Error> {
        let lang = if lang.is_empty() { LANG_EN } else { lang };

        if !VALID_LANGS.contains(&lang) {
            return Ok(());
        }

        let translations = load_translations(lang)?;

        let mut current = self.current_lang.write().unwrap();
        let mut trans = self.translations.write().unwrap();
        *current = lang.to_string();
        *trans = translations;

        Ok(())
    }
}

/// Load translations from a JSON file.
fn load_translations(lang: &str) -> Result<HashMap<String, String>, anyhow::Error> {
    let path = format!("src/i18n/translations/{lang}.json");
    let content = std::fs::read_to_string(&path)?;
    let translations: HashMap<String, String> = serde_json::from_str(&content)?;
    Ok(translations)
}

/// Get the global [`I18n`] singleton instance.
///
/// Uses `once_cell::sync::Lazy` for safe, thread-safe lazy initialization.
/// No unsafe code required.
#[must_use]
pub fn get_i18n() -> &'static I18n {
    static INSTANCE: Lazy<I18n> = Lazy::new(|| I18n {
        translations: RwLock::new(HashMap::new()),
        current_lang: RwLock::new(LANG_EN.to_string()),
    });
    &INSTANCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_translation() {
        let i18n = I18n {
            translations: RwLock::new(HashMap::from([
                ("hello".to_string(), "Hello".to_string()),
                ("world".to_string(), "World".to_string()),
            ])),
            current_lang: RwLock::new("en".to_string()),
        };
        assert_eq!(i18n.t("hello"), "Hello");
        assert_eq!(i18n.t("world"), "World");
        assert_eq!(i18n.t("missing"), "missing");
    }

    #[test]
    fn test_get_i18n_singleton() {
        let i18n = get_i18n();
        assert_eq!(i18n.current_lang(), LANG_EN);
    }
}
