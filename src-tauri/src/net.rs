use std::error::Error as _;
use std::time::Duration;

/// Providers such as Groq and OpenAI refuse uploads above 25 MB.
pub const MAX_UPLOAD_BYTES: usize = 24 * 1024 * 1024;

/// Builds a client with no overall deadline of its own; each request sets its own
/// timeout (see `stt_timeout` / `llm_timeout`) so long audio is not cut off at a fixed 90 s.
pub fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

/// Upload + transcription time grows with the clip: 60 s base, plus 1 s per 100 KB
/// (a slow 1 Mbit/s uplink moves ~125 KB/s), capped at 10 minutes.
pub fn stt_timeout(upload_bytes: usize) -> Duration {
    Duration::from_secs((60 + upload_bytes as u64 / 100_000).min(600))
}

/// Cleanup output is about as long as its input, so allow more time for long dictations.
pub fn llm_timeout(input_chars: usize) -> Duration {
    Duration::from_secs((60 + input_chars as u64 / 50).min(300))
}

/// Explains what actually went wrong while sending, instead of a generic "network error".
pub fn send_error(e: reqwest::Error, service: &str) -> String {
    if e.is_timeout() {
        return format!("The {service} took too long to answer (timed out). For a long recording, try again or record in shorter pieces.");
    }
    if e.is_connect() {
        return format!("Could not reach the {service}. Check your internet connection.");
    }
    // The innermost cause (e.g. "connection reset by peer") is the useful part.
    let mut cause = String::new();
    let mut src = e.source();
    while let Some(s) = src {
        cause = s.to_string();
        src = s.source();
    }
    let what = if e.is_body() || e.is_request() { "the connection was interrupted while sending" } else { "a network error occurred" };
    if cause.is_empty() {
        format!("Network error talking to the {service}: {what}.")
    } else {
        format!("Network error talking to the {service}: {what} ({cause}).")
    }
}

/// True for failures worth one quick retry (dropped connection), not timeouts or bad requests.
pub fn is_transient(e: &reqwest::Error) -> bool {
    !e.is_timeout() && !e.is_builder() && !e.is_decode() && (e.is_connect() || e.is_request() || e.is_body())
}

/// Turns a failed HTTP response into a friendly message (never includes the key).
pub async fn status_error(resp: reqwest::Response) -> String {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    let hint = match status.as_u16() {
        401 | 403 => "The API key was rejected. Check that it is correct and active.",
        404 => "Service address or model not found. Check the settings.",
        413 => "The recording is too large for the service. Record in shorter pieces.",
        429 => "Rate limit reached. Wait a moment and try again.",
        500..=599 => "The service returned an error (its servers may be having trouble). Try again in a moment.",
        _ => "The service returned an error.",
    };
    let detail: String = body.chars().take(300).collect();
    format!("{hint} (HTTP {}) {}", status.as_u16(), detail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_scale_with_size() {
        assert_eq!(stt_timeout(0), Duration::from_secs(60));
        assert!(stt_timeout(20_000_000) > stt_timeout(1_000_000));
        assert_eq!(stt_timeout(usize::MAX / 2), Duration::from_secs(600));
        assert!(llm_timeout(5000) > llm_timeout(100));
    }
}
