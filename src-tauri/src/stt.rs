use crate::net;
use crate::settings::{load_config, load_key, Config, KeyKind};
use serde::Deserialize;
use tauri::AppHandle;

/// Whisper's `prompt` biases spelling toward these words (it has a small token limit).
const MAX_PROMPT_CHARS: usize = 600;

/// Clips shorter than this are too little audio to trust language detection.
const SHORT_CLIP_SECS: f64 = 2.0;
/// Mean log-probability below this counts as a low-confidence transcription.
const LOW_CONFIDENCE: f64 = -0.8;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Segment {
    avg_logprob: Option<f64>,
}

/// `verbose_json` adds the detected language, clip length and per-segment confidence.
#[derive(Deserialize, Default)]
#[serde(default)]
struct Transcription {
    text: String,
    language: Option<String>,
    duration: Option<f64>,
    segments: Vec<Segment>,
}

impl Transcription {
    /// Mean segment log-probability (closer to 0 = more confident). None if unknown.
    fn confidence(&self) -> Option<f64> {
        let v: Vec<f64> = self.segments.iter().filter_map(|s| s.avg_logprob).collect();
        (!v.is_empty()).then(|| v.iter().sum::<f64>() / v.len() as f64)
    }
    fn language_code(&self) -> Option<String> {
        self.language.as_deref().and_then(language_code)
    }
}

/// The service reports languages by English name ("english"); map to ISO-639-1 codes.
fn language_code(name: &str) -> Option<String> {
    let n = name.trim().to_lowercase();
    if (2..=3).contains(&n.len()) {
        return Some(n);
    }
    let code = match n.as_str() {
        "english" => "en", "hindi" => "hi", "spanish" => "es", "french" => "fr", "german" => "de",
        "portuguese" => "pt", "italian" => "it", "dutch" => "nl", "russian" => "ru", "japanese" => "ja",
        "korean" => "ko", "chinese" => "zh", "arabic" => "ar", "turkish" => "tr", "polish" => "pl",
        "ukrainian" => "uk", "gujarati" => "gu", "marathi" => "mr", "bengali" => "bn", "tamil" => "ta",
        "telugu" => "te", "urdu" => "ur", "punjabi" => "pa", "indonesian" => "id", "vietnamese" => "vi",
        "thai" => "th", "swedish" => "sv", "greek" => "el", "hebrew" => "he",
        _ => return None,
    };
    Some(code.into())
}

/// Why an auto-detected result should be double-checked against the primary language,
/// or None when it can be trusted as is.
fn fallback_reason(first: &Transcription, primary: &str) -> Option<&'static str> {
    if first.language_code().as_deref() == Some(primary) {
        return None; // detection agrees with the primary language
    }
    if first.duration.is_some_and(|d| d < SHORT_CLIP_SECS) {
        return Some("very short clip");
    }
    if first.confidence().is_some_and(|c| c < LOW_CONFIDENCE) {
        return Some("low confidence");
    }
    None
}

/// Whether the primary-language retry should replace the auto-detected result. For a very short
/// clip the primary language wins unless it is clearly worse; otherwise it must be more confident.
fn prefer_retry(reason: &str, auto: &Transcription, retry: &Transcription) -> bool {
    if retry.text.trim().is_empty() {
        return false;
    }
    match (auto.confidence(), retry.confidence()) {
        (Some(a), Some(r)) if reason == "very short clip" => r >= a - 0.3,
        (Some(a), Some(r)) => r > a,
        _ => reason == "very short clip",
    }
}

fn build_prompt(vocab: &[String]) -> Option<String> {
    if vocab.is_empty() {
        return None;
    }
    let p = format!("Vocabulary: {}.", vocab.join(", "));
    Some(p.chars().take(MAX_PROMPT_CHARS).collect())
}

/// One transcription request. `language` None lets the service detect it.
async fn request(
    client: &reqwest::Client,
    key: &str,
    cfg: &Config,
    bytes: &[u8],
    prompt: Option<&str>,
    language: Option<&str>,
) -> Result<Transcription, String> {
    let timeout = net::stt_timeout(bytes.len());
    let url = format!("{}/audio/transcriptions", cfg.stt_base_url);
    // A multipart form can only be sent once, so it is rebuilt for the single retry.
    let build_form = || -> Result<reqwest::multipart::Form, String> {
        let part = reqwest::multipart::Part::bytes(bytes.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| e.to_string())?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("model", cfg.stt_model.clone())
            .text("response_format", "verbose_json")
            .text("temperature", "0");
        if let Some(l) = language {
            form = form.text("language", l.to_string());
        }
        if let Some(p) = prompt {
            form = form.text("prompt", p.to_string());
        }
        Ok(form)
    };

    let mut attempt = 0;
    let resp = loop {
        attempt += 1;
        let sent = client.post(&url).bearer_auth(key).timeout(timeout).multipart(build_form()?).send().await;
        match sent {
            Ok(r) => break r,
            Err(e) if attempt < 2 && net::is_transient(&e) => {
                let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(std::time::Duration::from_millis(1200))).await;
            }
            Err(e) => return Err(net::send_error(e, "transcription service")),
        }
    };
    if !resp.status().is_success() {
        return Err(net::status_error(resp).await);
    }
    let mut t: Transcription = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected response from the service: {}", e.without_url()))?;
    t.text = t.text.trim().to_string();
    Ok(t)
}

#[tauri::command]
pub async fn transcribe_last(app: AppHandle) -> Result<String, String> {
    let key = load_key(KeyKind::Stt)
        .ok_or("No transcription API key saved yet. Paste your key in Settings and click Save.")?;
    let cfg = load_config(&app);

    let wav_path = crate::audio::last_recording_path();
    let bytes = std::fs::read(&wav_path).map_err(|_| "No recording found. Record something first.")?;

    if bytes.len() > net::MAX_UPLOAD_BYTES {
        return Err(format!(
            "That recording is too large to upload ({:.0} MB; the limit is {:.0} MB). Record in shorter pieces.",
            bytes.len() as f32 / 1_048_576.0,
            net::MAX_UPLOAD_BYTES as f32 / 1_048_576.0
        ));
    }
    let vocab_prompt = build_prompt(&cfg.vocabulary_list());
    let client = net::client()?;
    let prompt = vocab_prompt.as_deref();

    if cfg.stt_language != "auto" {
        return Ok(request(&client, &key, &cfg, &bytes, prompt, Some(&cfg.stt_language)).await?.text);
    }

    // Auto-detect, with the primary language as the fallback for short/low-confidence clips.
    let first = request(&client, &key, &cfg, &bytes, prompt, None).await?;
    let detected = first.language_code();
    let Some(reason) = fallback_reason(&first, &cfg.primary_language) else {
        eprintln!("yapp: language auto: detected {detected:?} (primary {}), kept", cfg.primary_language);
        return Ok(first.text);
    };
    let retry = request(&client, &key, &cfg, &bytes, prompt, Some(&cfg.primary_language)).await?;
    let use_retry = prefer_retry(reason, &first, &retry);
    eprintln!(
        "yapp: language auto: detected {detected:?} ({reason}, {:.1}s, conf {:?}); retried as primary {} (conf {:?}) -> using {}",
        first.duration.unwrap_or(0.0),
        first.confidence(),
        cfg.primary_language,
        retry.confidence(),
        if use_retry { "primary" } else { "detected" }
    );
    Ok(if use_retry { retry.text } else { first.text })
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

    fn t(lang: &str, dur: f64, conf: f64) -> Transcription {
        Transcription {
            text: "x".into(),
            language: Some(lang.into()),
            duration: Some(dur),
            segments: vec![Segment { avg_logprob: Some(conf) }],
        }
    }

    #[test]
    fn fallback_only_for_short_or_unsure_disagreeing_clips() {
        assert_eq!(fallback_reason(&t("english", 1.0, -0.1), "en"), None); // agrees
        assert_eq!(fallback_reason(&t("hindi", 5.0, -0.2), "en"), None); // confident, long
        assert_eq!(fallback_reason(&t("hindi", 1.0, -0.2), "en"), Some("very short clip"));
        assert_eq!(fallback_reason(&t("hindi", 5.0, -1.2), "en"), Some("low confidence"));
    }

    #[test]
    fn retry_preference() {
        assert!(prefer_retry("very short clip", &t("hindi", 1.0, -0.5), &t("english", 1.0, -0.6)));
        assert!(!prefer_retry("very short clip", &t("hindi", 1.0, -0.2), &t("english", 1.0, -1.5)));
        assert!(prefer_retry("low confidence", &t("hindi", 5.0, -1.2), &t("english", 5.0, -0.4)));
        assert!(!prefer_retry("low confidence", &t("hindi", 5.0, -1.2), &t("english", 5.0, -1.4)));
    }
}
