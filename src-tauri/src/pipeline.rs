use crate::audio::{self, Recorder};
use crate::{insert, llm, settings, stats, stt};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[derive(Default)]
pub struct PipelineState {
    recorder: Mutex<Option<Recorder>>,
    processing: AtomicBool,
    /// Bumped on every pill change so a delayed hide never hides a newer pill.
    pub pill_generation: AtomicU64,
    /// True between a hotkey press and its release; Windows repeats presses while keys are held.
    key_down: AtomicBool,
    hotkey: Mutex<Option<Shortcut>>,
    hotkey_error: Mutex<Option<String>>,
}

#[derive(Serialize, Clone)]
struct StatePayload {
    /// "idle" | "recording" | "processing" | "error"
    state: &'static str,
    message: String,
    /// A few words for the pill when state is "error".
    short: Option<String>,
}

#[derive(Serialize, Clone, Default)]
struct ResultPayload {
    raw: String,
    clean: String,
    info: String,
    /// Set when transcription worked but cleanup failed.
    cleanup_error: Option<String>,
}

fn emit_state(app: &AppHandle, state: &'static str, message: impl Into<String>) {
    let message = message.into();
    crate::ui::sync_pill(app, state);
    let _ = app.emit("yapp://state", StatePayload { state, message, short: None });
}

fn emit_result(app: &AppHandle, r: ResultPayload) {
    let _ = app.emit("yapp://result", r);
}

// ---------- Hotkey ----------

pub fn register_hotkey(app: &AppHandle, text: &str) -> Result<(), String> {
    let shortcut: Shortcut = text
        .parse()
        .map_err(|e| format!("\"{text}\" is not a valid hotkey ({e}). Example: Ctrl+Space"))?;
    let state = app.state::<PipelineState>();
    let mut current = state.hotkey.lock().map_err(|_| "internal error")?;
    if *current == Some(shortcut) {
        return Ok(());
    }
    // Register the new one first so a failure leaves the old hotkey working.
    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| on_hotkey(app, event.state()))
        .map_err(|e| {
            format!("Could not use \"{text}\" as the hotkey ({e}). Another app may already be using it - try a different one.")
        })?;
    if let Some(old) = current.take() {
        let _ = app.global_shortcut().unregister(old);
    }
    *current = Some(shortcut);
    if let Ok(mut err) = state.hotkey_error.lock() {
        *err = None;
    }
    Ok(())
}

pub fn set_startup_hotkey_error(app: &AppHandle, message: String) {
    if let Ok(mut err) = app.state::<PipelineState>().hotkey_error.lock() {
        *err = Some(message);
    }
}

#[derive(Serialize)]
pub struct HotkeyStatus {
    error: Option<String>,
}

#[tauri::command]
pub fn get_hotkey_status(state: tauri::State<PipelineState>) -> HotkeyStatus {
    HotkeyStatus { error: state.hotkey_error.lock().ok().and_then(|e| e.clone()) }
}

fn on_hotkey(app: &AppHandle, key_state: ShortcutState) {
    let st = app.state::<PipelineState>();
    match key_state {
        ShortcutState::Released => st.key_down.store(false, Ordering::SeqCst),
        ShortcutState::Pressed => {
            if st.key_down.swap(true, Ordering::SeqCst) {
                return; // auto-repeat from holding the keys
            }
            let app = app.clone();
            tauri::async_runtime::spawn(async move { toggle(app).await });
        }
    }
}

// ---------- Toggle + pipeline ----------

async fn toggle(app: AppHandle) {
    let st = app.state::<PipelineState>();
    if st.processing.load(Ordering::SeqCst) {
        emit_state(&app, "processing", "Still working on the last recording - one moment.");
        return;
    }
    let taken = match st.recorder.lock() {
        Ok(mut slot) => slot.take(),
        Err(_) => return,
    };
    match taken {
        None => match Recorder::start() {
            Ok(rec) => {
                if let Ok(mut slot) = st.recorder.lock() {
                    *slot = Some(rec);
                }
                emit_state(&app, "recording", "Recording... press the hotkey again to stop.");
                stream_levels(app.clone());
            }
            Err(e) => fail(&app, e),
        },
        Some(rec) => {
            st.processing.store(true, Ordering::SeqCst);
            let _guard = ProcessingGuard(app.clone());
            run_pipeline(&app, rec).await;
        }
    }
}

/// While a recording is open, sends its loudness ~30 times a second for the pill's waveform.
fn stream_levels(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(33));
        let level = match app.state::<PipelineState>().recorder.lock() {
            Ok(slot) => slot.as_ref().map(|r| r.level()),
            Err(_) => None,
        };
        match level {
            Some(l) => {
                let _ = app.emit("yapp://level", l);
            }
            None => break,
        }
    });
}

/// Resets `processing` when the pipeline ends - even if it panics, in which case the
/// user gets a FAILED pill instead of a silently stuck app.
struct ProcessingGuard(AppHandle);

impl Drop for ProcessingGuard {
    fn drop(&mut self) {
        if std::thread::panicking() {
            fail(&self.0, "Something unexpected went wrong inside yapp.");
        }
        self.0.state::<PipelineState>().processing.store(false, Ordering::SeqCst);
        audio::delete_last_recording();
    }
}

/// Shows the FAILED pill and the full explanation in Settings. Nothing is inserted.
fn fail(app: &AppHandle, detail: impl Into<String>) {
    let detail = detail.into();
    let short = short_reason(&detail);
    crate::ui::sync_pill(app, "error");
    let _ = app.emit("yapp://state", StatePayload { state: "error", message: detail, short: Some(short) });
}

/// A few words for the pill, chosen from the (our own) error messages.
fn short_reason(detail: &str) -> String {
    let d = detail.to_lowercase();
    let table: &[(&str, &str)] = &[
        ("no microphone", "No microphone found"),
        ("disconnected", "Microphone disconnected"),
        ("privacy settings", "Microphone blocked - check Windows privacy settings"),
        ("could not open the microphone", "Can't use the microphone"),
        ("could not read microphone", "Can't use the microphone"),
        ("no audio was captured", "No audio captured"),
        ("minutes long", "Recording too long"),
        ("microphone looks silent", "Microphone seems silent"),
        ("no speech detected", "No speech detected"),
        ("api key saved", "No API key - open Settings"),
        ("key was rejected", "Invalid API key"),
        ("rate limit", "Rate limit reached - try again shortly"),
        ("could not reach", "No internet connection"),
        ("took too long", "Service timed out"),
        ("not found", "Model or address not found - check Settings"),
        ("too large", "Recording too large"),
        ("servers may be having trouble", "Service error - try again"),
        ("service returned an error", "Service error - try again"),
        ("network error", "Network error"),
        ("unexpected response", "Bad response from the service"),
        ("nothing was inserted", "Nothing to insert"),
        ("inserting", "Couldn't insert the text"),
    ];
    table
        .iter()
        .find(|(needle, _)| d.contains(needle))
        .map(|(_, short)| short.to_string())
        .unwrap_or_else(|| "Something went wrong".into())
}

async fn run_pipeline(app: &AppHandle, rec: Recorder) {
    emit_state(app, "processing", "Saving audio...");
    let recorded = match rec.stop() {
        Ok(r) => r,
        Err(e) => return fail(app, e),
    };
    let silent = recorded.peak < 0.01;
    let quiet = if silent { " WARNING: almost silent - check your microphone." } else { "" };
    let mut result = ResultPayload {
        info: format!(
            "Recorded {:.1}s, sent {:.1}s after trimming silence.{quiet}",
            recorded.original_seconds, recorded.seconds
        ),
        ..Default::default()
    };

    emit_state(app, "processing", "Transcribing...");
    let t0 = Instant::now();
    let transcript = stt::transcribe_last(app.clone()).await;
    audio::delete_last_recording(); // privacy default: audio is gone as soon as it has been sent
    let raw = match transcript {
        Ok(t) => t,
        Err(e) => return fail(app, e),
    };
    let stt_secs = t0.elapsed().as_secs_f32();
    result.raw = raw.clone();
    if raw.is_empty() {
        emit_result(app, result);
        return fail(
            app,
            if silent {
                "No speech detected - your microphone looks silent. Check it is the right one in Windows Settings > Sound > Input."
            } else {
                "No speech detected. Try again a little closer to the microphone."
            },
        );
    }
    emit_result(app, result.clone()); // show the raw text while cleanup runs

    emit_state(app, "processing", "Cleaning up...");
    let t1 = Instant::now();
    match llm::cleanup_text(app.clone(), raw).await {
        Ok(clean) => {
            let timing = format!("transcribed {stt_secs:.1}s + cleaned {:.1}s", t1.elapsed().as_secs_f32());
            result.clean = clean.clone();
            emit_result(app, result);

            if clean.is_empty() {
                return fail(app, "The cleanup returned no text, so nothing was inserted.");
            }
            let method = settings::load_config(app).insert_method;
            if method == "off" {
                stats::record(app, &clean, recorded.seconds);
                return emit_state(app, "idle", format!("Done ({timing}; insertion is off in Settings)."));
            }
            emit_state(app, "processing", "Inserting at the cursor...");
            let counted = clean.clone();
            let inserted = tauri::async_runtime::spawn_blocking(move || insert::insert(&clean, &method)).await;
            match inserted {
                Ok(Ok(how)) => {
                    stats::record(app, &counted, recorded.seconds);
                    emit_state(app, "idle", format!("Done ({timing}); text {how} at the cursor."));
                }
                Ok(Err(e)) => fail(app, format!("Inserting the text failed: {e}. The cleaned text is under Last dictation in Settings.")),
                Err(_) => fail(app, "Inserting the text crashed unexpectedly."),
            }
        }
        Err(e) => {
            result.cleanup_error = Some(e.clone());
            emit_result(app, result);
            fail(app, format!("Cleanup failed: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failures_get_short_pill_reasons() {
        let cases = [
            ("No microphone found. Plug one in", "No microphone found"),
            ("Could not start the microphone (is it blocked in Windows privacy settings?): x", "Microphone blocked - check Windows privacy settings"),
            ("The API key was rejected. Check that it is correct and active. (HTTP 401)", "Invalid API key"),
            ("Cleanup failed: Rate limit reached. Wait a moment and try again. (HTTP 429)", "Rate limit reached - try again shortly"),
            ("Could not reach the transcription service. Check your internet connection.", "No internet connection"),
            ("The cleanup service took too long to answer.", "Service timed out"),
            ("No transcription API key saved yet. Paste your key", "No API key - open Settings"),
            ("No speech detected - your microphone looks silent.", "Microphone seems silent"),
            ("No speech detected. Try again", "No speech detected"),
            ("The service returned an error. (HTTP 400)", "Service error - try again"),
            ("something odd", "Something went wrong"),
        ];
        for (detail, want) in cases {
            assert_eq!(short_reason(detail), want, "{detail}");
        }
    }

    #[test]
    fn hotkey_strings_parse() {
        for s in ["Ctrl+Space", "Ctrl+Alt+K", "Shift+F9", "F8", "Ctrl+Shift+Digit1"] {
            assert!(s.parse::<Shortcut>().is_ok(), "{s} should parse");
        }
        assert!("Ctrl+".parse::<Shortcut>().is_err());
    }
}