#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod llm;
mod models;
mod net;
mod pipeline;
mod prompt;
mod settings;
mod stt;

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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(pipeline::PipelineState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            let cfg = settings::load_config(&handle);
            if let Err(e) = pipeline::register_hotkey(&handle, &cfg.hotkey) {
                pipeline::set_startup_hotkey_error(&handle, e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            reveal_recording,
            pipeline::get_hotkey_status,
            settings::get_settings,
            settings::save_settings,
            models::check_models,
            stt::transcribe_last,
            llm::cleanup_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running Flow");
}
