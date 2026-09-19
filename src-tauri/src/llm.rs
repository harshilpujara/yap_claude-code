use crate::net;
use crate::prompt::{vocabulary_addendum, CLEANUP_SYSTEM_PROMPT};
use crate::settings::{load_config, load_llm_key};
use serde::Deserialize;
use serde_json::json;
use tauri::AppHandle;

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}
#[derive(Deserialize)]
struct Choice {
    message: Message,
}
#[derive(Deserialize)]
struct Message {
    content: Option<String>,
}

fn system_prompt(vocab: &[String]) -> String {
    let mut p = CLEANUP_SYSTEM_PROMPT.to_string();
    if !vocab.is_empty() {
        p.push_str(&vocabulary_addendum(vocab));
    }
    p
}

#[tauri::command]
pub async fn cleanup_text(app: AppHandle, raw: String) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(String::new());
    }
    let cfg = load_config(&app);
    let key = load_llm_key(&cfg)
        .ok_or("No cleanup API key saved yet. Paste your key in Settings and click Save.")?;

    let body = json!({
        "model": cfg.llm_model,
        "temperature": 0,
        "messages": [
            { "role": "system", "content": system_prompt(&cfg.vocabulary_list()) },
            { "role": "user", "content": raw }
        ]
    });

    let resp = net::client()?
        .post(format!("{}/chat/completions", cfg.llm_base_url))
        .bearer_auth(key)
        .json(&body)
        .send()
        .await
        .map_err(|e| net::send_error(e, "cleanup service"))?;

    if !resp.status().is_success() {
        return Err(net::status_error(resp).await);
    }
    let parsed: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("Unexpected response from the service: {}", e.without_url()))?;
    let text = parsed
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .unwrap_or_default();
    Ok(text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_includes_rules_and_vocabulary() {
        let base = system_prompt(&[]);
        assert!(base.contains("Do NOT add information"));
        assert!(base.contains("INSTRUCTING the formatting") && base.contains("keep the words as text"));
        assert!(!base.contains("Vocabulary:"));
        let with = system_prompt(&["Harshil".into()]);
        assert!(with.contains("Vocabulary:") && with.contains("Harshil"));
    }
}
