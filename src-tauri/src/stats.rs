//! Local usage stats for the Insights tab. Only counts are stored (words, seconds,
//! dictations per day) - never any transcript text - in `stats.json` in the app
//! config folder. Nothing is sent anywhere.

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

/// Typing speed assumed when estimating time saved.
const TYPING_WPM: f64 = 40.0;
/// Days of history sent to the heatmap (26 weeks plus the current partial week).
const HEATMAP_DAYS: i64 = 26 * 7 + 6;

static FILE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
struct Day {
    words: u64,
    dictations: u64,
    /// Seconds of audio sent for transcription (a fair stand-in for speaking time).
    seconds: f64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct Store {
    /// Keyed by local date, "YYYY-MM-DD".
    days: BTreeMap<String, Day>,
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("stats.json"))
}

fn load(app: &AppHandle) -> Store {
    path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save(app: &AppHandle, store: &Store) -> Result<(), String> {
    let p = path(app)?;
    let tmp = p.with_extension("json.tmp");
    let json = serde_json::to_string(store).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

pub fn count_words(text: &str) -> u64 {
    text.split_whitespace().count() as u64
}

/// Called once after each successful dictation. Failures never affect dictation.
pub fn record(app: &AppHandle, text: &str, audio_seconds: f32) {
    let words = count_words(text);
    if words == 0 {
        return;
    }
    let _guard = FILE_LOCK.lock();
    let mut store = load(app);
    let day = store.days.entry(Local::now().date_naive().to_string()).or_default();
    day.words += words;
    day.dictations += 1;
    day.seconds += audio_seconds.max(0.0) as f64;
    if save(app, &store).is_ok() {
        let _ = app.emit("yapp://stats", ());
    }
}

#[derive(Serialize, Debug, PartialEq)]
pub struct DayWords {
    date: String,
    words: u64,
}

#[derive(Serialize, Debug)]
pub struct StatsView {
    total_words: u64,
    dictations: u64,
    /// None until there is enough speech to measure.
    avg_wpm: Option<f64>,
    time_saved_seconds: f64,
    current_streak: u32,
    longest_streak: u32,
    typing_wpm: f64,
    days: Vec<DayWords>,
}

fn parse(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// (current, longest) runs of consecutive days with any words. The current streak is
/// still alive if today has nothing yet but yesterday did.
fn streaks(active: &[NaiveDate], today: NaiveDate) -> (u32, u32) {
    let mut dates: Vec<NaiveDate> = active.to_vec();
    dates.sort();
    dates.dedup();
    let mut longest = 0u32;
    let mut run = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for d in &dates {
        run = if prev.map_or(false, |p| p + Duration::days(1) == *d) { run + 1 } else { 1 };
        longest = longest.max(run);
        prev = Some(*d);
    }
    let mut cursor = if dates.contains(&today) { today } else { today - Duration::days(1) };
    let mut current = 0u32;
    while dates.contains(&cursor) {
        current += 1;
        cursor -= Duration::days(1);
    }
    (current, longest)
}

fn view(store: &Store, today: NaiveDate) -> StatsView {
    let total_words: u64 = store.days.values().map(|d| d.words).sum();
    let dictations: u64 = store.days.values().map(|d| d.dictations).sum();
    let seconds: f64 = store.days.values().map(|d| d.seconds).sum();
    let avg_wpm = (seconds >= 5.0 && total_words > 0).then(|| total_words as f64 / (seconds / 60.0));
    let typing_seconds = total_words as f64 / TYPING_WPM * 60.0;
    let active: Vec<NaiveDate> =
        store.days.iter().filter(|(_, d)| d.words > 0).filter_map(|(k, _)| parse(k)).collect();
    let (current_streak, longest_streak) = streaks(&active, today);
    let since = today - Duration::days(HEATMAP_DAYS);
    let days = store
        .days
        .iter()
        .filter(|(k, _)| parse(k).map_or(false, |d| d >= since))
        .map(|(k, d)| DayWords { date: k.clone(), words: d.words })
        .collect();
    StatsView {
        total_words,
        dictations,
        avg_wpm,
        time_saved_seconds: (typing_seconds - seconds).max(0.0),
        current_streak,
        longest_streak,
        typing_wpm: TYPING_WPM,
        days,
    }
}

#[tauri::command]
pub fn get_stats(app: AppHandle) -> StatsView {
    let _guard = FILE_LOCK.lock();
    view(&load(&app), Local::now().date_naive())
}

#[tauri::command]
pub fn reset_stats(app: AppHandle) -> Result<(), String> {
    let _guard = FILE_LOCK.lock();
    save(&app, &Store::default())?;
    let _ = app.emit("yapp://stats", ());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        parse(s).unwrap()
    }

    #[test]
    fn counts_words() {
        assert_eq!(count_words("  Hello,   world\nagain "), 3);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn streaks_current_and_longest() {
        let days = [d("2026-03-01"), d("2026-03-02"), d("2026-03-03"), d("2026-03-05"), d("2026-03-06")];
        assert_eq!(streaks(&days, d("2026-03-06")), (2, 3)); // today counts
        assert_eq!(streaks(&days, d("2026-03-07")), (2, 3)); // yesterday keeps it alive
        assert_eq!(streaks(&days, d("2026-03-08")), (0, 3)); // a gap breaks it
        assert_eq!(streaks(&[], d("2026-03-08")), (0, 0));
    }

    #[test]
    fn wpm_and_time_saved() {
        let mut s = Store::default();
        s.days.insert("2026-03-06".into(), Day { words: 200, dictations: 2, seconds: 60.0 });
        let v = view(&s, d("2026-03-06"));
        assert_eq!(v.total_words, 200);
        assert_eq!(v.dictations, 2);
        assert!((v.avg_wpm.unwrap() - 200.0).abs() < 1e-9);
        // typing 200 words at 40 wpm = 300 s, minus 60 s speaking
        assert!((v.time_saved_seconds - 240.0).abs() < 1e-9);
        assert_eq!(view(&Store::default(), d("2026-03-06")).avg_wpm, None);
    }
}
