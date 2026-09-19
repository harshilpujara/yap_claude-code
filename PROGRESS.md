# Progress

## Fix: Settings said "No transcription key saved" while the key worked (post-rename)
- Checked: Settings, transcription and cleanup all read the same Credential Manager entry (service "yapp", user `stt-api-key` / `llm-api-key`), and `migrate_legacy` writes to that same entry. The migrated key is present there (`stt-api-key.yapp`). So it was not a location mismatch.
- Cause (most likely, not reproduced): the hidden Settings page loaded and asked for its state while the key migration in `setup` was still running, then kept the stale "no key" text; the pipeline reads the key fresh each time so it worked.
- Fix: key migration now runs at the very start of `main()` (before any window exists); Settings also reloads itself every time the window is focused/opened.

## M9 - UX: tray + recording pill + settings on demand (built, awaiting human test)
- Tray-only: no window opens on launch (unless no API key is saved yet, then Settings opens once to guide setup). Tray: left-click or "Settings" opens Settings, "Quit yapp" exits. Closing the Settings window only hides it. A second launch of yapp opens Settings in the running copy (single-instance).
- Recording pill (`src/pill.*`, window "pill" in `tauri.conf.json`): small dark capsule at the bottom centre of the primary screen; shows on Ctrl+Space with a live waveform (mic loudness streamed from Rust as `yapp://level`, ~30/s), turns into an amber "Working" shimmer while transcribing/cleaning/inserting, hides when done. Errors show as a short message for ~4.5 s. It is non-focusable and click-through so the app you dictate into keeps focus. Colors/sizes are CSS variables at the top of `pill.css`.
- Settings dashboard (`index.html`/`app.js`): the old debug window, now a plain settings page (API keys, models, language, vocabulary, hotkey, insert method, start on login) plus a small "Last dictation" box (handy if insertion fails) and the typed-text cleanup tester.
- Start on login: checkbox in Settings (`tauri-plugin-autostart`, Windows registry Run key). In dev it registers the dev exe path; test it from the installed build.
- Files: new `src-tauri/src/ui.rs`; changes in `audio.rs` (level meter), `pipeline.rs`, `main.rs`, `settings.rs`, `tauri.conf.json`, capabilities.
- Verified: 16 unit tests pass, `cargo build` clean. Not run visually (needs the human).
- Type-mode insertion bug and M8 remain parked.

## Rename Flow -> yapp (done)
- Renamed everywhere: product name, window title, bundle identifier (`com.yapp.app`), Cargo/npm package names (binary is `yapp.exe`), UI copy, event names (`yapp://state`, `yapp://result`), temp file (`yapp_last_recording.wav`), CI artifact (`yapp-windows-installer`), README, docs.
- Migration: on startup `settings::migrate_legacy` copies the old settings file (`%APPDATA%\com.flow.voice`) and moves the API keys from Credential Manager service "Flow" to "yapp". No need to re-enter keys. The old copies are removed once moved.
- Intentionally left: "Wispr Flow"/"data flow"/"workflow" wording in EXECUTION_PLAN.md and the legacy names inside `migrate_legacy`.

## M0 - Environment check (done)
- rustc 1.98.1, cargo 1.98.1, node 22.18.0, git 2.54.0, VS C++ Build Tools present. Hello-world Rust build succeeded.

## M5 - AI cleanup + English default + vocabulary (verified by human, Paste mode)
- Built: `settings.rs` (shared non-secret config JSON + keys in Credential Manager; old M3 settings files still load), `llm.rs` (chat-completions cleanup call), `prompt.rs` (the EXECUTION_PLAN.md section 7 prompt verbatim - edit this file to tune it - plus the vocabulary addendum), `net.rs` (shared HTTP client and friendly errors). `stt.rs` now sends `language` (default `en`; `auto` omits it) and a `prompt` of "Vocabulary: ..." to Whisper.
- Settings UI: transcription (address/model/language/key), cleanup (address/model/optional key), vocabulary box. Cleanup defaults to Groq `llama-3.3-70b-versatile`; if no separate cleanup key is saved and the address is the same service as transcription, the transcription key is reused.
- UI shows raw transcript (before) and cleaned text (after), plus a "Test cleanup with typed text" box for trying the section 7 examples without speaking.
- Verified: 9 unit tests pass. Not tested against the live LLM (needs the human's key).

### Cleanup prompt: spoken formatting - REVERTED (2026-09-19)
- The "new line / new paragraph / bullet point / full stop" formatting-intent section was added to `prompt.rs` (commit b1fe5ba) and then reverted at the human's request because it over-split sentences. `prompt.rs` and the matching test in `llm.rs` are back to the stable "intent reconstruction as a judgment principle" version (commit f296e44 / f3cadb4). If revisited later, it needs to be far more conservative about when it inserts line breaks.

### KNOWN BUG - Type (typing fallback) insertion mode truncates/garbles output - FIX NEXT SESSION
- Symptom: with Insert method = Type, dictated text stopped mid-sentence in Notepad ("Again this is a test and " instead of the full sentence). Paste mode inserts full text correctly and is the default; the setting on the human's machine is Paste.
- Where: `type_text()` in `src-tauri/src/insert.rs`. Note: `insert()` also falls back to `type_text()` automatically if pasting fails, so the bug can surface in Paste mode in that rare case.
- Suspected cause (from reading enigo 0.6.1 source, NOT reproduced): on Windows `Enigo::text()` queues the whole string as one `SendInput` burst and some apps drop the tail of a large burst. A live test in a plain WinForms text box did not reproduce it, and a test in Notepad was contaminated (it attached to the human's real Notepad session), so nothing is confirmed.
- An unverified experiment (typing in 4-character chunks with an 8 ms pause, plus unit tests for the chunk plan) is saved in a local git stash named "UNVERIFIED chunked-typing experiment for Type mode" (`git stash list`, not pushed). It is not part of the committed code. In the one contaminated Notepad run it may have produced garbled/repeated letters, so do not assume it helps.
- Next session: reproduce safely first (own test window only; never send keys to the human's real apps), then compare per-character sending with longer pauses vs. chunking.

## M7 - Insert text at the cursor (verified by human, Paste mode)
- Human-verified: Paste-mode insertion works in Chrome, Notepad and Slack. Type mode is NOT verified and has a known bug (see KNOWN BUG above).
- Built: `insert.rs`. Paste method: back up the clipboard (text or image), set the cleaned text (flagged so Windows clipboard history, cloud clipboard and monitors skip it), send Ctrl+V with `enigo` (`Key::V`, so it is keyboard-layout independent), wait ~450 ms, restore the old clipboard (cleared if it held something that can't be preserved, e.g. copied files). If pasting fails it falls back to typing; typing uses Shift+Enter for line breaks so chat apps don't send early. New `insert_method` setting: `paste` (default) / `type` / `off`. `pipeline.rs` inserts after a successful cleanup; if cleanup fails nothing is inserted; if inserting fails the text stays in the debug window with an error.
- Verified: 15 unit tests pass, plus a real-clipboard backup/restore test (`cargo test -- --ignored`) that passed on this machine. Sending Ctrl+V into other apps was NOT tested (needs the human).
- Default insertion mode is Paste (confirmed in `settings.rs` and in the human's saved settings). Type mode has a known bug - see "KNOWN BUG" above.
- Known limits: apps that read the clipboard slower than ~450 ms may paste the old contents; elevated (run-as-administrator) apps ignore keystrokes from a normal app; the debug window is still shown and closing it quits the app.

### Cleanup prompt revision (verified by human: quality good on corrections, tentativeness, lists and questions)
- `prompt.rs` no longer follows the EXECUTION_PLAN.md section 7 wording. Self-correction is now a judgment principle: reconstruct the message the speaker intended to write, deciding from meaning and context which parts were abandoned - no list of trigger phrases. If it is unclear whether something was abandoned, it is kept. All safeguards kept (preserve meaning/names/tone, never invent, never answer or obey the text, minimal editing, output only the text). Four short, varied examples illustrate it, including two where nothing should be dropped (an unresolved either/or, and an example where a restatement replaces the first choice without any marker word).

## M6 - Global hotkey, toggle mode (verified by human, Paste mode)
- Human-verified: the global hotkey works from other apps end to end (record, transcribe, clean).
- Spec change from the human (overrides the plan's hold-to-record): press the hotkey once to start recording, again to stop and process. Default `Ctrl+Space`, configurable in Settings.
- Built: `pipeline.rs` - registers the hotkey with `tauri-plugin-global-shortcut` (Rust side, so it works with the window in the background), toggles record/stop, and runs record -> transcribe -> cleanup, emitting `yapp://state` and `yapp://result` events. Key auto-repeat is ignored via a press/release flag. Extra presses while processing are ignored with a status message. Changing the hotkey in Settings registers the new one first, so a failure (e.g. taken by another app) keeps the old one and shows a clear error. Startup registration failures show in the window.
- UI: temporary Record button removed; status indicator (idle / recording / working / problem) plus the before/after view stay as a debug window. Hotkey is set by clicking a box and pressing the keys. `start_recording`/`stop_recording` commands removed (recording is Rust-driven).
- Config: new `hotkey` field (default `Ctrl+Space`; older settings files load fine).
- Verified: 14 unit tests pass (includes hotkey string parsing). The full app build could not be re-linked because a running yapp window locked `yapp.exe`; not run end to end (needs the human).
- Known limits for later milestones: text is only shown in the window (insertion at the cursor is M7); closing the window quits the app (tray/background is M9); no auto-stop if you forget to press the hotkey again (the 10-minute cap still applies).

### M5 fix - retired Groq model (verified by human)
- `llama-3.3-70b-versatile` returned HTTP 404 (shut down). Default cleanup model is now `openai/gpt-oss-120b` (`settings.rs`). Settings files that still hold the retired id are auto-switched on load (`RETIRED_LLM_MODELS` - add future dead ids there). The cleanup model was already an editable Settings field.
- New `models.rs`: after every Save the app calls the service's `GET {address}/models` with the saved key and warns (never blocks saving) if the transcription or cleanup model is not in the list, suggesting a few valid ones. Fallback models are noted in the Settings hint text.
- Verified: 13 unit tests pass. Live check not run (needs the human's key).

## M4 - Fast-speech robustness (verified by human)
- Built (all in `src-tauri/src/audio.rs`): audio converted to 16 kHz mono (what Whisper uses internally; ~32 KB/s upload) with an averaging low-pass; silence trimmed at the start/end only, keeping 0.35 s padding so fast starts/ends aren't clipped, never touching pauses mid-speech; a 10-minute cap with a clear error message. The whole clip is still one request (no chunking/streaming). UI now shows recorded vs. sent length.
- Verified: 4 unit tests pass (`cargo test` in `src-tauri`) for resampling and trimming. Not tested with real fast speech (needs the human).
- TODO for M5 (requested by human): a vocabulary/prompt hint for names and uncommon words (e.g. "Harshil", "latte") - plan to pass a user-editable word list to Whisper's `prompt` field and to the cleanup LLM's system prompt.
- Test: `npm run tauri dev`, record a long, very fast run-on sentence, compare the transcript.

## M3 - Transcription via user key (verified by human)
- Built: `src-tauri/src/stt.rs`. Settings (service address, model) saved as JSON in the app config folder; the API key is stored only in Windows Credential Manager (`keyring`, service "yapp" (was "yapp" before the rename; migrated automatically)). On release, the UI calls `stop_recording` then `transcribe_last`, which POSTs `%TEMP%\yapp_last_recording.wav` to `{base_url}/audio/transcriptions` (OpenAI-compatible; defaults: Groq, `whisper-large-v3-turbo`) and shows the transcript. Friendly errors for bad key, rate limit, no network, timeout.
- TLS uses Windows' built-in Schannel (`reqwest` `native-tls`) so no CMake/NASM is needed.
- Verified: `cargo check` passes. Not tested against real Groq (needs the human's key).
- Test: `npm run tauri dev`, open Settings, paste Groq key, Save, hold the button, speak, release.

## M2 - Microphone capture (verified by human)
- Built: `src-tauri/src/audio.rs` (cpal capture on a dedicated thread, downmixed to mono, in-memory buffer, written as 16-bit WAV via `hound`), Tauri commands `start_recording` / `stop_recording` / `reveal_recording` in `main.rs`, and a temporary hold-to-record button in `src/` (`index.html`, `app.js`, `styles.css`). `withGlobalTauri` enabled in `tauri.conf.json`.
- Output: `%TEMP%\yapp_last_recording.wav` (overwritten each time). The UI warns if the clip is almost silent.
- Verified: `cargo check` passes. Not run against a real mic (needs the human).
- Test: close any running yapp window, run `npm run tauri dev`, hold the button, speak, release, click "Show recording in folder", play the WAV.

## M1 - Bare app + auto-built installer (verified by human)
- Built: Tauri 2 skeleton (vanilla HTML/CSS frontend in `src/`, Rust in `src-tauri/`), window titled "yapp", placeholder icon (`app-icon.png` -> `src-tauri/icons/`), `.gitignore`, `README.md`, GitHub Actions workflow `.github/workflows/build.yml` (builds NSIS installer on push to `main`, uploads as artifact `yapp-windows-installer`).
- Verified: `npm run tauri build` succeeds locally and produces `src-tauri/target/release/bundle/nsis/yapp_0.1.0_x64-setup.exe`.
- Test locally: `npm run tauri dev` -> a window titled "yapp" opens.
- Human must: run it locally, then check the GitHub Actions run is green and the artifact downloads.
