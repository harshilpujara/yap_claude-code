#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod insert;
mod llm;
mod models;
mod net;
mod pipeline;
mod prompt;
mod settings;
mod stats;
mod stt;
mod ui;

fn main() {
    settings::migrate_legacy_keys();
    // Privacy: never leave audio behind, even after a crash.
    audio::delete_last_recording();
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
            // First-time users (no key yet, or welcome dialog not seen) start in Settings.
            if !settings::has_any_key() || !settings::onboarding_seen(&handle) {
                ui::show_settings(&handle);
            }
            ui::refresh_autostart(&handle);
            ui::watch_for_resume(&handle);
            let cfg = settings::load_config(&handle);
            if let Err(e) = pipeline::register_hotkey(&handle, &cfg.hotkey) {
                pipeline::set_startup_hotkey_error(&handle, e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pipeline::get_hotkey_status,
            settings::get_settings,
            stats::get_stats,
            stats::reset_stats,
            ui::open_url,
            ui::get_autostart,
            ui::set_autostart,
            settings::save_settings,
            settings::get_onboarding_seen,
            settings::set_onboarding_seen,
            settings::log_onboarding_decision,
            models::check_models,
            stt::transcribe_last,
            llm::cleanup_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running yapp");
}
