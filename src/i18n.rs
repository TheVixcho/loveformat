use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

static LOCALES_JSON: &str = include_str!("../locales.json");
static CURRENT_LANG: RwLock<String> = RwLock::new(String::new());
static LOCALES_MAP: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

fn get_locales() -> &'static HashMap<String, HashMap<String, String>> {
    LOCALES_MAP.get_or_init(|| serde_json::from_str(LOCALES_JSON).unwrap_or_default())
}

pub fn available_languages() -> Vec<(String, String)> {
    let locales = get_locales();
    let mut langs = Vec::new();
    if locales.contains_key("en") {
        langs.push(("en".to_string(), "English".to_string()));
    }
    if locales.contains_key("es") {
        langs.push(("es".to_string(), "Español".to_string()));
    }
    for (k, map) in locales {
        if k != "en" && k != "es" {
            let name = map
                .get("language_name")
                .cloned()
                .unwrap_or_else(|| k.clone());
            langs.push((k.clone(), name));
        }
    }
    langs
}

pub fn set_language(lang: &str) {
    if let Ok(mut lock) = CURRENT_LANG.write() {
        *lock = lang.to_string();
    }
}

pub fn current_language() -> String {
    if let Ok(lock) = CURRENT_LANG.read() {
        if !lock.is_empty() {
            return lock.clone();
        }
    }
    "en".to_string()
}

pub fn t(key: &str) -> String {
    let lang = current_language();
    let locales = get_locales();

    if let Some(map) = locales.get(&lang) {
        if let Some(val) = map.get(key) {
            return val.clone();
        }
    }

    if lang != "en" {
        if let Some(map) = locales.get("en") {
            if let Some(val) = map.get(key) {
                return val.clone();
            }
        }
    }

    key.to_string()
}

pub fn t_fmt(key: &str, args: &[&str]) -> String {
    let template = t(key);
    let mut result = template;
    for arg in args {
        if let Some(idx) = result.find("{}") {
            result.replace_range(idx..idx + 2, arg);
        }
    }
    result
}

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("loveformat").join("config.json")
}

pub fn load_saved_language() -> Option<String> {
    let p = config_path();
    if p.is_file() {
        if let Ok(content) = fs::read_to_string(&p) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                return val
                    .get("language")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

pub fn save_saved_language(lang: &str) {
    let p = config_path();
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let data = serde_json::json!({ "language": lang });
    let _ = fs::write(p, data.to_string());
}
