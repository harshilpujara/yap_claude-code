//! The two visible surfaces: the recording pill and the on-demand settings window,
//! plus the system tray icon that is the app's only permanent presence.

use crate::pipeline::PipelineState;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};
use tauri_plugin_autostart::ManagerExt;

const PILL: &str = "pill";
const SETTINGS: &str = "main";
/// Gap between the pill and the bottom of the screen's usable area, in logical pixels.
const PILL_BOTTOM_MARGIN: f64 = 28.0;

pub fn show_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        // The page goes back to the Dashboard whenever it is opened from the tray.
        let _ = app.emit("yapp://opened", ());
    }
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit yapp", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &quit])?;
    let mut tray = TrayIconBuilder::with_id("yapp")
        .tooltip("yapp")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                show_settings(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Closing the settings window only hides it; the app keeps running in the tray.
pub fn hide_settings_on_close(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(SETTINGS) {
        let hidden = w.clone();
        w.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = hidden.hide();
            }
        });
    }
}

/// The pill never takes focus or mouse clicks, so the app being dictated into keeps both.
pub fn prepare_pill(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PILL) {
        let _ = w.set_ignore_cursor_events(true);
    }
}

fn place_pill(w: &WebviewWindow) {
    let Ok(Some(monitor)) = w.primary_monitor() else { return };
    let Ok(size) = w.outer_size() else { return };
    let area = monitor.work_area();
    let margin = PILL_BOTTOM_MARGIN * monitor.scale_factor();
    let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
    let y = area.position.y + area.size.height as i32 - size.height as i32 - margin as i32;
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

/// Shows/hides the pill to match the pipeline state. Errors stay visible a few seconds.
pub fn sync_pill(app: &AppHandle, state: &str) {
    let Some(w) = app.get_webview_window(PILL) else { return };
    let st = app.state::<PipelineState>();
    let generation = st.pill_generation.fetch_add(1, Ordering::SeqCst) + 1;
    if state != "idle" {
        place_pill(&w);
        let _ = w.show();
    }
    match state {
        "recording" | "processing" => {}
        _ => {
            let linger = if state == "error" { Duration::from_millis(5500) } else { Duration::from_millis(250) };
            let app = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(linger);
                let st = app.state::<PipelineState>();
                if st.pill_generation.load(Ordering::SeqCst) == generation {
                    if let Some(w) = app.get_webview_window(PILL) {
                        let _ = w.hide();
                    }
                }
            });
        }
    }
}

/// Opens a web link in the user's default browser (never inside the app window).
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !url.starts_with("https://") || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("Only https:// links can be opened.".into());
    }
    // explorer.exe hands the address to the default browser without going through a shell.
    std::process::Command::new("explorer").arg(&url).spawn().map(|_| ()).map_err(|e| e.to_string())
}

// ---------- Start on login ----------

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    result.map_err(|e| format!("Could not change the start-on-login setting: {e}"))
}
