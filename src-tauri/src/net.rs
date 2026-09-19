use std::time::Duration;

pub fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())
}

pub fn send_error(e: reqwest::Error, service: &str) -> String {
    if e.is_timeout() {
        format!("The {service} took too long to answer.")
    } else if e.is_connect() {
        format!("Could not reach the {service}. Check your internet connection.")
    } else {
        format!("Network error: {}", e.without_url())
    }
}

/// Turns a failed HTTP response into a friendly message (never includes the key).
pub async fn status_error(resp: reqwest::Response) -> String {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    let hint = match status.as_u16() {
        401 | 403 => "The API key was rejected. Check that it is correct and active.",
        404 => "Service address or model not found. Check the settings.",
        413 => "The request is too large for the service.",
        429 => "Rate limit reached. Wait a moment and try again.",
        _ => "The service returned an error.",
    };
    let detail: String = body.chars().take(300).collect();
    format!("{hint} (HTTP {}) {}", status.as_u16(), detail)
}
