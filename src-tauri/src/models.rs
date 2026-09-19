use crate::net;
use crate::settings::{load_config, load_key, load_llm_key, KeyKind};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Deserialize)]
struct ModelList {
    data: Vec<ModelInfo>,
}
#[derive(Deserialize)]
struct ModelInfo {
    id: String,
}

#[derive(Serialize)]
pub struct ModelCheck {
    /// Empty when everything was verified.
    pub warnings: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Purpose {
    Speech,
    Chat,
}

fn is_speech(id: &str) -> bool {
    id.contains("whisper")
}

fn is_chat(id: &str) -> bool {
    let id = id.to_lowercase();
    !["whisper", "guard", "tts", "playai", "orpheus", "embed", "safeguard"]
        .iter()
        .any(|bad| id.contains(bad))
}

/// None if `wanted` is available; otherwise a message listing a few valid alternatives.
fn check_model(ids: &[String], wanted: &str, purpose: Purpose, label: &str) -> Option<String> {
    if ids.iter().any(|i| i == wanted) {
        return None;
    }
    let suggestions: Vec<&str> = ids
        .iter()
        .map(String::as_str)
        .filter(|i| match purpose {
            Purpose::Speech => is_speech(i),
            Purpose::Chat => is_chat(i),
        })
        .take(6)
        .collect();
    let list = if suggestions.is_empty() {
        String::new()
    } else {
        format!(" Models available to your key include: {}.", suggestions.join(", "))
    };
    Some(format!(
        "The {label} model \"{wanted}\" was not found on this service.{list}"
    ))
}

async fn fetch_ids(base_url: &str, key: &str, service: &str) -> Result<Vec<String>, String> {
    let resp = net::client()?
        .get(format!("{base_url}/models"))
        .bearer_auth(key)
        .send()
        .await
        .map_err(|e| net::send_error(e, service))?;
    if !resp.status().is_success() {
        return Err(net::status_error(resp).await);
    }
    let list: ModelList = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected model list from the {service}: {}", e.without_url()))?;
    Ok(list.data.into_iter().map(|m| m.id).collect())
}

/// Checks the saved model names against each service's live model list.
/// Problems are reported as warnings; this never blocks saving.
#[tauri::command]
pub async fn check_models(app: AppHandle) -> ModelCheck {
    let cfg = load_config(&app);
    let mut warnings = Vec::new();

    match load_key(KeyKind::Stt) {
        None => warnings.push("Could not verify the transcription model: no transcription key saved.".into()),
        Some(key) => match fetch_ids(&cfg.stt_base_url, &key, "transcription service").await {
            Ok(ids) => warnings.extend(check_model(&ids, &cfg.stt_model, Purpose::Speech, "transcription")),
            Err(e) => warnings.push(format!("Could not verify the transcription model: {e}")),
        },
    }

    match load_llm_key(&cfg) {
        None => warnings.push("Could not verify the cleanup model: no cleanup key saved.".into()),
        Some(key) => match fetch_ids(&cfg.llm_base_url, &key, "cleanup service").await {
            Ok(ids) => warnings.extend(check_model(&ids, &cfg.llm_model, Purpose::Chat, "cleanup")),
            Err(e) => warnings.push(format!("Could not verify the cleanup model: {e}")),
        },
    }

    ModelCheck { warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> Vec<String> {
        ["openai/gpt-oss-120b", "openai/gpt-oss-20b", "whisper-large-v3-turbo", "meta-llama/llama-guard-4-12b"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn available_model_passes() {
        assert!(check_model(&ids(), "openai/gpt-oss-120b", Purpose::Chat, "cleanup").is_none());
    }

    #[test]
    fn missing_chat_model_lists_chat_models_only() {
        let m = check_model(&ids(), "llama-3.3-70b-versatile", Purpose::Chat, "cleanup").unwrap();
        assert!(m.contains("openai/gpt-oss-20b") && !m.contains("whisper") && !m.contains("guard"));
    }

    #[test]
    fn missing_speech_model_lists_whisper_models() {
        let m = check_model(&ids(), "whisper-old", Purpose::Speech, "transcription").unwrap();
        assert!(m.contains("whisper-large-v3-turbo") && !m.contains("gpt-oss"));
    }
}
