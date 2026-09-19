#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod insert;
mod llm;
mod models;
mod net;
mod pipeline;
mod prompt;
mod settings;
mod stt;
mod ui;

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
    settings::migrate_legacy_keys();
    tauri::Builder::default()
        // A second launch (e.g. from the Start menu) just opens Settings in the running app.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| ui::show_settings(app)))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(pipeline::PipelineState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            settings::migrate_legacy(&handle);
            ui::hide_settings_on_close(&handle);
            ui::prepare_pill(&handle);
            ui::setup_tray(&handle)?;
            // Nothing to see until a key exists, so guide first-time users to Settings.
            if !settings::has_any_key() {
                ui::show_settings(&handle);
            }
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
            ui::get_autostart,
            ui::set_autostart,
            settings::save_settings,
            models::check_models,
            stt::transcribe_last,
            llm::cleanup_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running yapp");
}
