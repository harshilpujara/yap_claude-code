# Progress

## M0 - Environment check (done)
- rustc 1.98.1, cargo 1.98.1, node 22.18.0, git 2.54.0, VS C++ Build Tools present. Hello-world Rust build succeeded.

## M2 - Microphone capture (built, awaiting human check)
- Built: `src-tauri/src/audio.rs` (cpal capture on a dedicated thread, downmixed to mono, in-memory buffer, written as 16-bit WAV via `hound`), Tauri commands `start_recording` / `stop_recording` / `reveal_recording` in `main.rs`, and a temporary hold-to-record button in `src/` (`index.html`, `app.js`, `styles.css`). `withGlobalTauri` enabled in `tauri.conf.json`.
- Output: `%TEMP%\flow_last_recording.wav` (overwritten each time). The UI warns if the clip is almost silent.
- Verified: `cargo check` passes. Not run against a real mic (needs the human).
- Test: close any running Flow window, run `npm run tauri dev`, hold the button, speak, release, click "Show recording in folder", play the WAV.

## M1 - Bare app + auto-built installer (verified by human)
- Built: Tauri 2 skeleton (vanilla HTML/CSS frontend in `src/`, Rust in `src-tauri/`), window titled "Flow", placeholder icon (`app-icon.png` -> `src-tauri/icons/`), `.gitignore`, `README.md`, GitHub Actions workflow `.github/workflows/build.yml` (builds NSIS installer on push to `main`, uploads as artifact `flow-windows-installer`).
- Verified: `npm run tauri build` succeeds locally and produces `src-tauri/target/release/bundle/nsis/Flow_0.1.0_x64-setup.exe`.
- Test locally: `npm run tauri dev` -> a window titled "Flow" opens.
- Human must: run it locally, then check the GitHub Actions run is green and the artifact downloads.
