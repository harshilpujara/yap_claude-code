use arboard::{Clipboard, ImageData, SetExtWindows};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::borrow::Cow;
use std::thread::sleep;
use std::time::Duration;

/// Gives the clipboard time to settle before we press Ctrl+V.
const BEFORE_PASTE: Duration = Duration::from_millis(60);
/// Gives the target app time to read the clipboard before we restore the old contents.
const BEFORE_RESTORE: Duration = Duration::from_millis(450);

enum Backup {
    Text(String),
    Image { width: usize, height: usize, bytes: Vec<u8> },
}

fn backup_clipboard(cb: &mut Clipboard) -> Option<Backup> {
    if let Ok(t) = cb.get_text() {
        return Some(Backup::Text(t));
    }
    if let Ok(img) = cb.get_image() {
        return Some(Backup::Image {
            width: img.width,
            height: img.height,
            bytes: img.bytes.into_owned(),
        });
    }
    None // empty, or a format we cannot preserve (e.g. copied files)
}

fn restore_clipboard(cb: &mut Clipboard, backup: Option<Backup>) {
    let _ = match backup {
        Some(Backup::Text(t)) => cb.set_text(t),
        Some(Backup::Image { width, height, bytes }) => {
            cb.set_image(ImageData { width, height, bytes: Cow::Owned(bytes) })
        }
        // Don't leave the dictated text behind on the clipboard.
        None => cb.clear(),
    };
}

/// Puts `text` on the clipboard (hidden from Windows clipboard history and cloud sync),
/// runs `action`, then restores whatever was there before.
fn with_temporary_clipboard(
    text: &str,
    action: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| format!("Could not open the clipboard: {e}"))?;
    let backup = backup_clipboard(&mut cb);
    cb.set()
        .exclude_from_history()
        .exclude_from_cloud()
        .exclude_from_monitoring()
        .text(text.to_owned())
        .map_err(|e| format!("Could not set the clipboard: {e}"))?;
    sleep(BEFORE_PASTE);
    let result = action();
    sleep(BEFORE_RESTORE);
    restore_clipboard(&mut cb, backup);
    result
}

fn new_enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|e| format!("Could not simulate the keyboard: {e}"))
}

fn press_ctrl_v() -> Result<(), String> {
    let mut enigo = new_enigo()?;
    let err = |e| format!("Could not send Ctrl+V: {e}");
    enigo.key(Key::Control, Direction::Press).map_err(err)?;
    let click = enigo.key(Key::V, Direction::Click);
    // Always let go of Ctrl, even if the V click failed.
    let release = enigo.key(Key::Control, Direction::Release);
    click.map_err(err)?;
    release.map_err(err)
}

fn paste(text: &str) -> Result<(), String> {
    with_temporary_clipboard(text, press_ctrl_v)
}

/// Inserts `text` at the cursor of the focused app by pasting it. `method` is "paste".
/// There is deliberately no key-by-key typing: it was unreliable in some apps (it could
/// cut text off), and quietly falling back to it could insert garbled text. If pasting
/// fails, the caller shows an error and nothing is inserted.
/// Returns a short description of what happened.
pub fn insert(text: &str, _method: &str) -> Result<String, String> {
    paste(text)
        .map(|()| "pasted".to_string())
        .map_err(|e| format!("Pasting failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Touches the real clipboard (restores it afterwards); run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn clipboard_is_restored_after_temporary_use() {
        let mut cb = Clipboard::new().unwrap();
        let users_own = backup_clipboard(&mut cb);
        cb.set_text("yapp-test-original").unwrap();
        let mut seen = String::new();
        let outcome = with_temporary_clipboard("yapp-test-dictated", || {
            seen = Clipboard::new().unwrap().get_text().unwrap();
            Ok(())
        });
        let after = cb.get_text().unwrap();
        restore_clipboard(&mut cb, users_own);
        outcome.unwrap();
        assert_eq!(seen, "yapp-test-dictated");
        assert_eq!(after, "yapp-test-original");
    }
}
