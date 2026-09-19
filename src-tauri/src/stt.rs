use crate::net;
use crate::settings::{load_config, load_key, KeyKind};
use serde::Deserialize;
use tauri::AppHandle;

/// Whisper's `prompt` biases spelling toward these words (it has a small token limit).
const MAX_PROMPT_CHARS: usize = 600;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

fn build_prompt(vocab: &[String]) -> Option<String> {
    if vocab.is_empty() {
        return None;
    }
    let p = format!("Vocabulary: {}.", vocab.join(", "));
    Some(p.chars().take(MAX_PROMPT_CHARS).collect())
}

#[tauri::command]
pub async fn transcribe_last(app: AppHandle) -> Result<String, String> {
    let key = load_key(KeyKind::Stt)
        .ok_or("No transcription API key saved yet. Paste your key in Settings and click Save.")?;
    let cfg = load_config(&app);

    let wav_path = std::env::temp_dir().join("flow_last_recording.wav");
    let bytes = std::fs::read(&wav_path).map_err(|_| "No recording found. Record something first.")?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let mut form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", cfg.stt_model.clone())
        .text("response_format", "json")
        .text("temperature", "0");
    if cfg.stt_language != "auto" {
        form = form.text("language", cfg.stt_language.clone());
    }
    if let Some(prompt) = build_prompt(&cfg.vocabulary_list()) {
        form = form.text("prompt", prompt);
    }

    let resp = net::client()?
        .post(format!("{}/audio/transcriptions", cfg.stt_base_url))
        .bearer_auth(key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| net::send_error(e, "transcription service"))?;

    if !resp.status().is_success() {
        return Err(net::status_error(resp).await);
    }
    let parsed: TranscriptionResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected response from the service: {}", e.without_url()))?;
    Ok(parsed.text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_lists_vocabulary() {
        let p = build_prompt(&["Harshil".into(), "latte".into()]).unwrap();
        assert_eq!(p, "Vocabulary: Harshil, latte.");
        assert!(build_prompt(&[]).is_none());
    }
}
