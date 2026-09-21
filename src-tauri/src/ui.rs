//! The two visible surfaces: the recording pill and the on-demand settings window,
//! plus the system tray icon that is the app's only permanent presence.

use crate::pipeline::PipelineState;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Listener, Manager, PhysicalPosition, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

const PILL: &str = "pill";
const SETTINGS: &str = "main";
/// The pill window sits flush on the bottom edge of the usable screen area; the gap above
/// that edge is drawn by the page itself, so the pill can slide out of / into that edge.
/// How long the page's slide-out animation takes before the window is really hidden.
const SLIDE_OUT: Duration = Duration::from_millis(380);

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
    // The pill page answers `yapp://pill-ping` so we can tell a dead page from a live one.
    let handle = app.clone();
    app.listen("yapp://pill-pong", move |event| {
        let n: u64 = event.payload().trim().parse().unwrap_or(0);
        handle.state::<PipelineState>().pong_seq.fetch_max(n, Ordering::SeqCst);
    });
}

/// Destroys the pill window and creates a fresh one from the config in tauri.conf.json.
/// Blocking: call from a background thread, never from an event handler or sync command.
fn rebuild_pill(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(PILL) {
        let _ = w.destroy();
    }
    for _ in 0..100 {
        if app.get_webview_window(PILL).is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let Some(cfg) = app.config().app.windows.iter().find(|c| c.label == PILL).cloned() else { return };
    match WebviewWindowBuilder::from_config(app, &cfg).and_then(|b| b.build()) {
        Ok(w) => {
            let _ = w.set_ignore_cursor_events(true);
        }
        Err(e) => eprintln!("yapp: could not recreate the pill window: {e}"),
    }
}

/// Before each dictation: makes sure the pill window exists and its page is answering.
/// After sleep/wake (or a WebView crash) the page can be gone while the window object is
/// still there, which used to mean "pipeline runs, but nothing is drawn".
pub async fn ensure_pill_alive(app: &AppHandle) {
    let st = app.state::<PipelineState>();
    if app.get_webview_window(PILL).is_some() {
        let n = st.ping_seq.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = app.emit("yapp://pill-ping", n);
        for _ in 0..30 {
            if st.pong_seq.load(Ordering::SeqCst) >= n {
                return;
            }
            let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(10))).await;
        }
    }
    let handle = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || rebuild_pill(&handle)).await;
    // A new page needs a moment to load before it can hear events.
    for _ in 0..30 {
        let n = st.ping_seq.fetch_add(1, Ordering::SeqCst) + 1;
        let _ = app.emit("yapp://pill-ping", n);
        let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(100))).await;
        if st.pong_seq.load(Ordering::SeqCst) >= n {
            return;
        }
    }
}

/// Detects the PC waking from sleep: this thread sleeps a few seconds at a time, so if far
/// more wall-clock time than that has passed, the whole process was frozen (or the clock jumped).
pub fn watch_for_resume(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        const TICK: Duration = Duration::from_secs(3);
        let mut last = std::time::SystemTime::now();
        loop {
            std::thread::sleep(TICK);
            let now = std::time::SystemTime::now();
            let gap = now.duration_since(last).unwrap_or(Duration::ZERO);
            last = now;
            if gap > Duration::from_secs(15) {
                // Give the network, audio devices and graphics a moment to come back.
                std::thread::sleep(Duration::from_millis(1500));
                crate::pipeline::recover_after_resume(&app);
                rebuild_pill(&app);
                last = std::time::SystemTime::now();
            }
        }
    });
}

fn place_pill(w: &WebviewWindow) {
    let Ok(Some(monitor)) = w.primary_monitor() else { return };
    let Ok(size) = w.outer_size() else { return };
    let area = monitor.work_area();
    let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
    let y = area.position.y + area.size.height as i32 - size.height as i32;
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

/// Shows/hides the pill to match the pipeline state. Errors stay visible a few seconds.
pub fn sync_pill(app: &AppHandle, state: &str) {
    let Some(w) = app.get_webview_window(PILL) else { return };
    let st = app.state::<PipelineState>();
    let generation = st.pill_generation.fetch_add(1, Ordering::SeqCst) + 1;
    if state != "idle" {
        place_pill(&w);
        let _ = w.set_always_on_top(true); // the OS can drop this across sleep or display changes
        if let Err(e) = w.show() {
            eprintln!("yapp: could not show the pill: {e}");
        }
    }
    match state {
        "recording" | "processing" => {}
        _ => {
            let linger = if state == "error" { Duration::from_millis(5500) } else { Duration::from_millis(250) };
            let app = app.clone();
            std::thread::spawn(move || {
                let still_current = |app: &AppHandle| {
                    app.state::<PipelineState>().pill_generation.load(Ordering::SeqCst) == generation
                };
                std::thread::sleep(linger);
                if !still_current(&app) {
                    return;
                }
                // Let the page slide the pill down out of view, then hide the window.
                let _ = app.emit("yapp://pill-hide", ());
                std::thread::sleep(SLIDE_OUT);
                if still_current(&app) {
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

/// The Windows "Run" entry stores the path of the exe that created it. A toggle switched on
/// from a dev build (or an older install location) leaves a path that no longer exists, and
/// the plugin still reports it as enabled. Rewriting the entry from the running exe fixes that.
#[cfg(not(debug_assertions))]
pub fn refresh_autostart(app: &AppHandle) {
    let launcher = app.autolaunch();
    if launcher.is_enabled().unwrap_or(false) {
        if let Err(e) = launcher.enable() {
            eprintln!("yapp: could not refresh the start-on-login entry: {e}");
        }
    }
}

#[cfg(debug_assertions)]
pub fn refresh_autostart(_app: &AppHandle) {
    // Dev builds must not write their temporary path into the user's login items.
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launcher = app.autolaunch();
    let failed = |e: tauri_plugin_autostart::Error| format!("Could not change the start-on-login setting: {e}");
    if enabled {
        // Clear any stale entry first so the new one always points at this exe.
        let _ = launcher.disable();
        launcher.enable().map_err(failed)?;
        if !launcher.is_enabled().unwrap_or(false) {
            return Err("Windows did not keep the start-on-login entry. Try again, or check Task Manager > Startup apps.".into());
        }
        Ok(())
    } else {
        launcher.disable().map_err(failed)
    }
}
