#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod stt;

use audio::{Recorder, RecordingResult};
use std::sync::Mutex;

#[derive(Default)]
struct AppState {
    recorder: Mutex<Option<Recorder>>,
}

#[tauri::command]
fn start_recording(state: tauri::State<AppState>) -> Result<(), String> {
    let mut slot = state.recorder.lock().map_err(|_| "internal error")?;
    if slot.is_some() {
        return Err("Already recording.".into());
    }
    *slot = Some(Recorder::start()?);
    Ok(())
}

#[tauri::command]
fn stop_recording(state: tauri::State<AppState>) -> Result<RecordingResult, String> {
    let recorder = state
        .recorder
        .lock()
        .map_err(|_| "internal error")?
        .take()
        .ok_or("Not recording.")?;
    recorder.stop()
}

#[tauri::command]
fn reveal_recording(path: String) -> Result<(), String> {
    let temp = std::env::temp_dir();
    if !std::path::Path::new(&path).starts_with(&temp) {
        return Err("Refusing to open a file outside the temp folder.".into());
    }
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            reveal_recording,
            stt::get_stt_settings,
            stt::save_stt_settings,
            stt::transcribe_last
        ])
        .run(tauri::generate_context!())
        .expect("error while running Flow");
}
