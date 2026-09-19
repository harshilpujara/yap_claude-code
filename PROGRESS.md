# Progress

## M0 - Environment check (done)
- rustc 1.98.1, cargo 1.98.1, node 22.18.0, git 2.54.0, VS C++ Build Tools present. Hello-world Rust build succeeded.

## M3 - Transcription via user key (built, awaiting human check)
- Built: `src-tauri/src/stt.rs`. Settings (service address, model) saved as JSON in the app config folder; the API key is stored only in Windows Credential Manager (`keyring`, service "Flow"). On release, the UI calls `stop_recording` then `transcribe_last`, which POSTs `%TEMP%\flow_last_recording.wav` to `{base_url}/audio/transcriptions` (OpenAI-compatible; defaults: Groq, `whisper-large-v3-turbo`) and shows the transcript. Friendly errors for bad key, rate limit, no network, timeout.
- TLS uses Windows' built-in Schannel (`reqwest` `native-tls`) so no CMake/NASM is needed.
- Verified: `cargo check` passes. Not tested against real Groq (needs the human's key).
- Test: `npm run tauri dev`, open Settings, paste Groq key, Save, hold the button, speak, release.

## M2 - Microphone capture (verified by human)
- Built: `src-tauri/src/audio.rs` (cpal capture on a dedicated thread, downmixed to mono, in-memory buffer, written as 16-bit WAV via `hound`), Tauri commands `start_recording` / `stop_recording` / `reveal_recording` in `main.rs`, and a temporary hold-to-record button in `src/` (`index.html`, `app.js`, `styles.css`). `withGlobalTauri` enabled in `tauri.conf.json`.
- Output: `%TEMP%\flow_last_recording.wav` (overwritten each time). The UI warns if the clip is almost silent.
- Verified: `cargo check` passes. Not run against a real mic (needs the human).
- Test: close any running Flow window, run `npm run tauri dev`, hold the button, speak, release, click "Show recording in folder", play the WAV.

## M1 - Bare app + auto-built installer (verified by human)
- Built: Tauri 2 skeleton (vanilla HTML/CSS frontend in `src/`, Rust in `src-tauri/`), window titled "Flow", placeholder icon (`app-icon.png` -> `src-tauri/icons/`), `.gitignore`, `README.md`, GitHub Actions workflow `.github/workflows/build.yml` (builds NSIS installer on push to `main`, uploads as artifact `flow-windows-installer`).
- Verified: `npm run tauri build` succeeds locally and produces `src-tauri/target/release/bundle/nsis/Flow_0.1.0_x64-setup.exe`.
- Test locally: `npm run tauri dev` -> a window titled "Flow" opens.
- Human must: run it locally, then check the GitHub Actions run is green and the artifact downloads.
