use crate::net;
use crate::prompt::{shortcuts_addendum, vocabulary_addendum, CLEANUP_SYSTEM_PROMPT};
use crate::settings::{load_config, load_llm_key, Shortcut};
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

fn system_prompt(vocab: &[String], shortcuts: &[Shortcut]) -> String {
    let mut p = CLEANUP_SYSTEM_PROMPT.to_string();
    if !vocab.is_empty() {
        p.push_str(&vocabulary_addendum(vocab));
    }
    if !shortcuts.is_empty() {
        p.push_str(&shortcuts_addendum(shortcuts));
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
            { "role": "system", "content": system_prompt(&cfg.vocabulary_list(), &cfg.shortcuts) },
            { "role": "user", "content": raw }
        ]
    });

    let resp = net::client()?
        .post(format!("{}/chat/completions", cfg.llm_base_url))
        .bearer_auth(key)
        .timeout(net::llm_timeout(raw.chars().count()))
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
    Ok(tidy_lines(&text))
}

/// Trims each line's trailing spaces (models sometimes add Markdown "hard break" spaces)
/// and keeps at most one blank line in a row, so pasted text has no stray whitespace.
fn tidy_lines(text: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for line in text.trim().lines() {
        let line = line.trim_end();
        if line.is_empty() && out.last().is_some_and(|l| l.is_empty()) {
            continue;
        }
        out.push(line);
    }
    out.join("
")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_includes_rules_and_vocabulary() {
        let base = system_prompt(&[], &[]);
        assert!(base.contains("Do NOT add information"));
        assert!(!base.contains("Vocabulary:") && !base.contains("Voice shortcuts:"));
        let with = system_prompt(&["Harshil".into()], &[]);
        assert!(with.contains("Vocabulary:") && with.contains("Harshil"));
        let sc = Shortcut { trigger: "sign off".into(), expansion: "Regards, Harshil".into() };
        let with = system_prompt(&[], &[sc]);
        assert!(with.contains("Voice shortcuts:") && with.contains("Regards, Harshil"));
    }

    #[test]
    fn tidy_lines_strips_trailing_spaces_and_extra_blanks() {
        assert_eq!(tidy_lines("  Hi Sam,  \r\n\r\n\r\n\nThanks  \nBye \n"), "Hi Sam,\n\nThanks\nBye");
    }

    /// Runs real dictations through the live cleanup model with the saved key and prints
    /// the results, so prompt changes can be judged by eye and by the asserts below.
    /// Run: cargo test live_cleanup -- --ignored --nocapture
    #[test]
    #[ignore]
    fn live_cleanup() {
        use crate::settings::{load_key, KeyKind};
        let key = load_key(KeyKind::Llm).or_else(|| load_key(KeyKind::Stt)).expect("no saved API key");
        // (input, must contain, must NOT contain)
        let cases: &[(&str, &[&str], &[&str])] = &[
            ("hi Sam new paragraph thanks for the update on the launch I read it this morning and the timeline looks fine to me new line one question though is the budget approved new paragraph thanks new line Harshil",
             &["Hi Sam", "\n\n", "Harshil", "budget approved"], &["new line", "new paragraph"]),
            ("dear team new line we launched the new line of products on Monday and sales were solid new paragraph the returns were higher than expected though so let's meet Wednesday no Thursday to go through them new paragraph regards new line Meera",
             &["Dear team", "new line of products", "Thursday", "returns", "Meera", "\n\n"], &["Wednesday"]),
            ("please add a new line item to the invoice for the extra hours and start a new paragraph in section two",
             &["new line item", "new paragraph in section two"], &["\n"]),
            ("what time is the meeting new line and who is coming",
             &["meeting", "coming"], &["new line"]),
            ("I'll book the 9am flight, no wait the 11am one, the 9am connection is too tight, and I'll take the window seat if there is one, and I need to tell Sam about the change",
             &["11am", "window seat", "Sam"], &["9am flight"]),
            ("so the demo went okay, the login page was slow though, and Priya asked about pricing which we didn't have an answer for, oh and we should send the deck tomorrow",
             &["login", "Priya", "pricing", "deck"], &["\n"]),
            ("the server crashed twice last night, I think it's the memory leak we saw before, Arjun should check the logs, and I'll write the incident report",
             &["twice", "memory leak", "Arjun", "incident report"], &["\n"]),
            ("we could ship it Thursday or Friday I honestly don't know yet",
             &["Thursday", "Friday"], &["\n"]),
        ];
        let client = reqwest::Client::new();
        for (input, must, must_not) in cases {
            let body = json!({"model": "openai/gpt-oss-120b", "temperature": 0, "messages": [
                {"role": "system", "content": system_prompt(&[], &[])}, {"role": "user", "content": input}]});
            let out = tauri::async_runtime::block_on(async {
                for _ in 0..6 {
                    let v: serde_json::Value = client.post("https://api.groq.com/openai/v1/chat/completions")
                        .bearer_auth(&key).json(&body).send().await.unwrap().json().await.unwrap();
                    if let Some(t) = v["choices"][0]["message"]["content"].as_str() {
                        return tidy_lines(t);
                    }
                    eprintln!("retrying after: {v}"); // e.g. a rate limit
                    std::thread::sleep(std::time::Duration::from_secs(8));
                }
                panic!("service kept failing");
            });
            println!("IN : {input}\nOUT:\n{out}\n-----");
            for m in *must { assert!(out.contains(m), "missing {m:?} for input {input:?}"); }
            for m in *must_not { assert!(!out.contains(m), "unexpected {m:?} for input {input:?}"); }
        }
    }
}
