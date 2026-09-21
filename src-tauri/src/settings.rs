use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const KEYRING_SERVICE: &str = "yapp";
// Names used before the app was renamed from "Flow" to "yapp".
const LEGACY_KEYRING_SERVICE: &str = "Flow";
const LEGACY_IDENTIFIER: &str = "com.flow.voice";

// Defaults suggest Groq; any OpenAI-compatible service works.
const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_STT_MODEL: &str = "whisper-large-v3-turbo";
const DEFAULT_LLM_MODEL: &str = "openai/gpt-oss-120b";
// Model ids we previously defaulted to that the service has since shut down.
const RETIRED_LLM_MODELS: &[&str] = &["llama-3.3-70b-versatile"];
const MAX_VOCAB_CHARS: usize = 1000;

/// Non-secret settings, stored as JSON in the app config folder.
/// (`alias` keeps settings saved by earlier milestones loading correctly.)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(alias = "base_url")]
    pub stt_base_url: String,
    #[serde(alias = "model")]
    pub stt_model: String,
    /// ISO-639-1 code such as "en", or "auto" to let the service detect it.
    pub stt_language: String,
    /// Used when `stt_language` is "auto": biases detection and is the fallback for very short
    /// or low-confidence clips, where detection is unreliable.
    pub primary_language: String,
    pub llm_base_url: String,
    pub llm_model: String,
    /// Names and uncommon words, separated by commas or new lines.
    pub vocabulary: String,
    /// Global toggle hotkey, e.g. "Ctrl+Space".
    pub hotkey: String,
    /// How the final text reaches the cursor: "paste" or "off" (show only).
    /// ("type" was removed - see PROGRESS.md; a saved "type" is treated as "paste".)
    pub insert_method: String,
    /// Voice shortcuts: spoken trigger phrases and the text they expand to.
    pub shortcuts: Vec<Shortcut>,
}

#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
#[serde(default)]
pub struct Shortcut {
    pub trigger: String,
    pub expansion: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stt_base_url: DEFAULT_BASE_URL.into(),
            stt_model: DEFAULT_STT_MODEL.into(),
            stt_language: "en".into(),
            primary_language: "en".into(),
            llm_base_url: DEFAULT_BASE_URL.into(),
            llm_model: DEFAULT_LLM_MODEL.into(),
            vocabulary: String::new(),
            hotkey: "Ctrl+Space".into(),
            insert_method: "paste".into(),
            shortcuts: Vec::new(),
        }
    }
}

impl Config {
    pub fn vocabulary_list(&self) -> Vec<String> {
        self.vocabulary
            .split(|c| c == ',' || c == '\n' || c == '\r')
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

const MAX_SHORTCUTS: usize = 100;
const MAX_TRIGGER_CHARS: usize = 100;
const MAX_EXPANSION_CHARS: usize = 2000;

/// Trims, drops rows with an empty trigger or expansion, and drops repeated triggers.
fn clean_shortcuts(list: Vec<Shortcut>) -> Vec<Shortcut> {
    let mut out: Vec<Shortcut> = Vec::new();
    for s in list {
        let trigger: String = s.trigger.trim().chars().take(MAX_TRIGGER_CHARS).collect();
        let expansion: String = s.expansion.trim().chars().take(MAX_EXPANSION_CHARS).collect();
        if trigger.is_empty() || expansion.is_empty() {
            continue;
        }
        if out.iter().any(|o| o.trigger.eq_ignore_ascii_case(&trigger)) {
            continue;
        }
        out.push(Shortcut { trigger, expansion });
        if out.len() >= MAX_SHORTCUTS {
            break;
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
pub enum KeyKind {
    Stt,
    Llm,
}

impl KeyKind {
    fn user(self) -> &'static str {
        match self {
            KeyKind::Stt => "stt-api-key",
            KeyKind::Llm => "llm-api-key",
        }
    }
}

fn key_entry(kind: KeyKind) -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, kind.user())
        .map_err(|e| format!("Could not access Windows Credential Manager: {e}"))
}

pub fn load_key(kind: KeyKind) -> Option<String> {
    key_entry(kind).ok()?.get_password().ok().filter(|k| !k.is_empty())
}

pub fn has_any_key() -> bool {
    load_key(KeyKind::Stt).is_some() || load_key(KeyKind::Llm).is_some()
}

fn host_of(url: &str) -> &str {
    url.split("://").nth(1).unwrap_or(url).split('/').next().unwrap_or("")
}

/// The cleanup key: its own key if saved, otherwise the transcription key when
/// both point at the same service (e.g. Groq for both).
pub fn load_llm_key(cfg: &Config) -> Option<String> {
    load_key(KeyKind::Llm).or_else(|| {
        if host_of(&cfg.llm_base_url) == host_of(&cfg.stt_base_url) {
            load_key(KeyKind::Stt)
        } else {
            None
        }
    })
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

/// One-time move of data saved under the old "Flow" name. Only fills in what the
/// new name does not have yet. Runs before any window exists so the Settings page
/// can never load while it is half done.
pub fn migrate_legacy(app: &AppHandle) {
    migrate_legacy_settings_file(app);
    migrate_legacy_keys();
}

fn migrate_legacy_settings_file(app: &AppHandle) {
    if let Ok(dir) = app.path().app_config_dir() {
        let new_file = dir.join("settings.json");
        let old_file = dir.with_file_name(LEGACY_IDENTIFIER).join("settings.json");
        if !new_file.exists() && old_file.exists() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::copy(&old_file, &new_file);
        }
    }
}

/// Moves API keys from Credential Manager service "Flow" to "yapp" (same user names).
pub fn migrate_legacy_keys() {
    for kind in [KeyKind::Stt, KeyKind::Llm] {
        let Ok(old) = keyring::Entry::new(LEGACY_KEYRING_SERVICE, kind.user()) else { continue };
        let Ok(secret) = old.get_password() else { continue };
        if load_key(kind).is_none() && !secret.is_empty() {
            if key_entry(kind).and_then(|e| e.set_password(&secret).map_err(|e| e.to_string())).is_err() {
                continue;
            }
        }
        let _ = old.delete_credential();
    }
}

pub fn load_config(app: &AppHandle) -> Config {
    let mut c: Config = config_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    if c.insert_method == "type" {
        c.insert_method = "paste".into();
    }
    if RETIRED_LLM_MODELS.contains(&c.llm_model.as_str()) {
        c.llm_model = DEFAULT_LLM_MODEL.into();
    }
    c
}

#[derive(Serialize)]
pub struct SettingsView {
    #[serde(flatten)]
    config: Config,
    has_stt_key: bool,
    has_llm_key: bool,
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> SettingsView {
    SettingsView {
        config: load_config(&app),
        has_stt_key: load_key(KeyKind::Stt).is_some(),
        has_llm_key: load_key(KeyKind::Llm).is_some(),
    }
}

fn apply_key(kind: KeyKind, key: Option<String>) -> Result<(), String> {
    match key.as_deref().map(str::trim) {
        Some("") => {
            let _ = key_entry(kind)?.delete_credential();
        }
        Some(k) => key_entry(kind)?
            .set_password(k)
            .map_err(|e| format!("Could not save the key securely: {e}"))?,
        None => {}
    }
    Ok(())
}

fn clean_url(url: &str, what: &str) -> Result<String, String> {
    let url = url.trim().trim_end_matches('/').to_string();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err(format!("The {what} address must start with https://"));
    }
    Ok(url)
}

/// For each key: None keeps the stored key, Some("") removes it, Some(text) replaces it.
#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    config: Config,
    stt_key: Option<String>,
    llm_key: Option<String>,
) -> Result<(), String> {
    let mut c = config;
    c.stt_base_url = clean_url(&c.stt_base_url, "transcription service")?;
    c.llm_base_url = clean_url(&c.llm_base_url, "cleanup service")?;
    c.stt_model = c.stt_model.trim().to_string();
    c.llm_model = c.llm_model.trim().to_string();
    if c.stt_model.is_empty() || c.llm_model.is_empty() {
        return Err("Please enter a model name for both services.".into());
    }
    c.stt_language = c.stt_language.trim().to_lowercase();
    let is_code = |l: &str| (2..=3).contains(&l.len()) && l.chars().all(|ch| ch.is_ascii_lowercase());
    if c.stt_language != "auto" && !is_code(&c.stt_language) {
        return Err("Language must be a short code like \"en\", or \"auto\".".into());
    }
    c.primary_language = c.primary_language.trim().to_lowercase();
    if !is_code(&c.primary_language) {
        return Err("Primary language must be a short code like \"en\".".into());
    }
    c.vocabulary = c.vocabulary.trim().chars().take(MAX_VOCAB_CHARS).collect();
    c.shortcuts = clean_shortcuts(c.shortcuts);
    c.hotkey = c.hotkey.trim().to_string();
    if c.hotkey.is_empty() {
        return Err("Please choose a hotkey.".into());
    }
    if !["paste", "off"].contains(&c.insert_method.as_str()) {
        return Err("Insert method must be paste or off.".into());
    }
    crate::pipeline::register_hotkey(&app, &c.hotkey)?;

    let json = serde_json::to_string_pretty(&c).map_err(|e| e.to_string())?;
    std::fs::write(config_path(&app)?, json).map_err(|e| e.to_string())?;
    apply_key(KeyKind::Stt, stt_key)?;
    apply_key(KeyKind::Llm, llm_key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_settings_file_still_loads() {
        let c: Config =
            serde_json::from_str(r#"{"base_url":"https://x.test/v1","model":"m"}"#).unwrap();
        assert_eq!(c.stt_base_url, "https://x.test/v1");
        assert_eq!(c.stt_model, "m");
        assert_eq!(c.stt_language, "en");
    }

    #[test]
    fn vocabulary_splits_on_commas_and_lines() {
        let c = Config { vocabulary: "Harshil, latte\nSarah,, ".into(), ..Config::default() };
        assert_eq!(c.vocabulary_list(), vec!["Harshil", "latte", "Sarah"]);
    }

    #[test]
    fn default_llm_model_is_not_retired() {
        assert!(!RETIRED_LLM_MODELS.contains(&DEFAULT_LLM_MODEL));
    }

    #[test]
    fn host_comparison() {
        assert_eq!(host_of("https://api.groq.com/openai/v1"), "api.groq.com");
    }
}

// ---------- First-run onboarding ----------

/// A marker file, separate from the settings and API keys, so the welcome dialog depends on
/// "has this person seen it?" and not on whether a key happens to be saved already.
fn onboarding_marker(app: &AppHandle) -> Option<PathBuf> {
    Some(app.path().app_config_dir().ok()?.join("onboarding_seen"))
}

/// Solely whether the marker file exists; an unresolvable config folder counts as "absent".
pub fn onboarding_seen(app: &AppHandle) -> bool {
    onboarding_marker(app).map(|p| p.exists()).unwrap_or(false)
}

#[tauri::command]
pub fn get_onboarding_seen(app: AppHandle) -> bool {
    let seen = onboarding_seen(&app);
    let marker = onboarding_marker(&app).map(|p| p.display().to_string()).unwrap_or_else(|| "<no config folder>".into());
    eprintln!("yapp: onboarding_seen={seen} (marker: {marker})");
    seen
}

/// Frontend reports its show/skip decision here so it lands in the same log as the check above.
#[tauri::command]
pub fn log_onboarding_decision(show: bool, reason: String) {
    eprintln!("yapp: onboarding {} - {reason}", if show { "SHOW" } else { "SKIP" });
}

#[tauri::command]
pub fn set_onboarding_seen(app: AppHandle) -> Result<(), String> {
    let path = onboarding_marker(&app).ok_or("No config folder.")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, "1").map_err(|e| e.to_string())
}
