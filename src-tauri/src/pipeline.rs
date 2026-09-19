use crate::audio::Recorder;
use crate::{insert, llm, settings, stt};
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
}

#[derive(Serialize, Clone, Default)]
struct ResultPayload {
    raw: String,
    clean: String,
    info: String,
    /// Set when transcription worked but cleanup failed.
    cleanup_error: Option<String>,
    recording_path: String,
}

fn emit_state(app: &AppHandle, state: &'static str, message: impl Into<String>) {
    let message = message.into();
    crate::ui::sync_pill(app, state);
    let _ = app.emit("yapp://state", StatePayload { state, message });
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
            Err(e) => emit_state(&app, "error", e),
        },
        Some(rec) => {
            st.processing.store(true, Ordering::SeqCst);
            run_pipeline(&app, rec).await;
            st.processing.store(false, Ordering::SeqCst);
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

async fn run_pipeline(app: &AppHandle, rec: Recorder) {
    emit_state(app, "processing", "Saving audio...");
    let recorded = match rec.stop() {
        Ok(r) => r,
        Err(e) => return emit_state(app, "error", e),
    };
    let quiet = if recorded.peak < 0.01 { " WARNING: almost silent - check your microphone." } else { "" };
    let mut result = ResultPayload {
        info: format!(
            "Recorded {:.1}s, sent {:.1}s after trimming silence.{quiet}",
            recorded.original_seconds, recorded.seconds
        ),
        recording_path: recorded.path,
        ..Default::default()
    };

    emit_state(app, "processing", "Transcribing...");
    let t0 = Instant::now();
    let raw = match stt::transcribe_last(app.clone()).await {
        Ok(t) => t,
        Err(e) => return emit_state(app, "error", e),
    };
    let stt_secs = t0.elapsed().as_secs_f32();
    result.raw = raw.clone();
    if raw.is_empty() {
        result.info.push_str(" No speech detected.");
        emit_result(app, result);
        return emit_state(app, "idle", format!("Done (transcribed in {stt_secs:.1}s, nothing to clean)."));
    }
    emit_result(app, result.clone()); // show the raw text while cleanup runs

    emit_state(app, "processing", "Cleaning up...");
    let t1 = Instant::now();
    match llm::cleanup_text(app.clone(), raw).await {
        Ok(clean) => {
            let timing = format!("transcribed {stt_secs:.1}s + cleaned {:.1}s", t1.elapsed().as_secs_f32());
            result.clean = clean.clone();
            emit_result(app, result);

            let method = settings::load_config(app).insert_method;
            if clean.is_empty() || method == "off" {
                let why = if clean.is_empty() { "nothing to insert" } else { "insertion is off in Settings" };
                return emit_state(app, "idle", format!("Done ({timing}; {why})."));
            }
            emit_state(app, "processing", "Inserting at the cursor...");
            let inserted = tauri::async_runtime::spawn_blocking(move || insert::insert(&clean, &method)).await;
            match inserted {
                Ok(Ok(how)) => emit_state(app, "idle", format!("Done ({timing}); text {how} at the cursor.")),
                Ok(Err(e)) => emit_state(app, "error", format!("Cleaned text is above, but inserting it failed: {e}")),
                Err(_) => emit_state(app, "error", "Inserting the text crashed unexpectedly."),
            }
        }
        Err(e) => {
            result.cleanup_error = Some(e.clone());
            emit_result(app, result);
            emit_state(app, "error", format!("Cleanup failed: {e}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_strings_parse() {
        for s in ["Ctrl+Space", "Ctrl+Alt+K", "Shift+F9", "F8", "Ctrl+Shift+Digit1"] {
            assert!(s.parse::<Shortcut>().is_ok(), "{s} should parse");
        }
        assert!("Ctrl+".parse::<Shortcut>().is_err());
    }
}