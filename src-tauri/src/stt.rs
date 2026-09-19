use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const KEYRING_SERVICE: &str = "Flow";
const KEYRING_STT_USER: &str = "stt-api-key";

// Defaults suggest Groq; any OpenAI-compatible transcription endpoint works.
const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_MODEL: &str = "whisper-large-v3-turbo";

#[derive(Serialize, Deserialize, Clone)]
pub struct SttConfig {
    pub base_url: String,
    pub model: String,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.into(),
            model: DEFAULT_MODEL.into(),
        }
    }
}

#[derive(Serialize)]
pub struct SttSettingsView {
    pub base_url: String,
    pub model: String,
    pub has_key: bool,
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

fn load_config(app: &AppHandle) -> SttConfig {
    config_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn key_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_STT_USER)
        .map_err(|e| format!("Could not access Windows Credential Manager: {e}"))
}

fn load_key() -> Option<String> {
    key_entry().ok()?.get_password().ok().filter(|k| !k.is_empty())
}

#[tauri::command]
pub fn get_stt_settings(app: AppHandle) -> SttSettingsView {
    let c = load_config(&app);
    SttSettingsView {
        base_url: c.base_url,
        model: c.model,
        has_key: load_key().is_some(),
    }
}

/// `api_key` = None keeps the stored key; Some("") removes it.
#[tauri::command]
pub fn save_stt_settings(
    app: AppHandle,
    base_url: String,
    model: String,
    api_key: Option<String>,
) -> Result<(), String> {
    let base_url = base_url.trim().trim_end_matches('/').to_string();
    let model = model.trim().to_string();
    if !(base_url.starts_with("https://") || base_url.starts_with("http://")) {
        return Err("The endpoint must start with https://".into());
    }
    if model.is_empty() {
        return Err("Please enter a model name.".into());
    }
    let json = serde_json::to_string_pretty(&SttConfig { base_url, model })
        .map_err(|e| e.to_string())?;
    std::fs::write(config_path(&app)?, json).map_err(|e| e.to_string())?;

    match api_key.as_deref().map(str::trim) {
        Some("") => {
            let _ = key_entry()?.delete_credential();
        }
        Some(k) => key_entry()?
            .set_password(k)
            .map_err(|e| format!("Could not save the key securely: {e}"))?,
        None => {}
    }
    Ok(())
}

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

#[tauri::command]
pub async fn transcribe_last(app: AppHandle) -> Result<String, String> {
    let key = load_key().ok_or("No API key saved yet. Paste your key in Settings and click Save.")?;
    let cfg = load_config(&app);

    let wav_path = std::env::temp_dir().join("flow_last_recording.wav");
    let bytes = std::fs::read(&wav_path).map_err(|_| "No recording found. Record something first.")?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", cfg.model)
        .text("response_format", "json")
        .text("temperature", "0");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client
        .post(format!("{}/audio/transcriptions", cfg.base_url))
        .bearer_auth(key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                "The transcription service took too long to answer.".to_string()
            } else if e.is_connect() {
                "Could not reach the transcription service. Check your internet connection.".to_string()
            } else {
                format!("Network error: {}", e.without_url())
            }
        })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let hint = match status.as_u16() {
            401 | 403 => "The API key was rejected. Check that it is correct and active.",
            404 => "Endpoint or model not found. Check the settings.",
            413 => "The recording is too large for the service.",
            429 => "Rate limit reached. Wait a moment and try again.",
            _ => "The service returned an error.",
        };
        let detail: String = body.chars().take(300).collect();
        return Err(format!("{hint} (HTTP {}) {}", status.as_u16(), detail));
    }

    let parsed: TranscriptionResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected response from the service: {}", e.without_url()))?;
    Ok(parsed.text.trim().to_string())
}
