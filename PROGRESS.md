# Progress

## M0 - Environment check (done)
- rustc 1.98.1, cargo 1.98.1, node 22.18.0, git 2.54.0, VS C++ Build Tools present. Hello-world Rust build succeeded.

## M5 - AI cleanup + English default + vocabulary (built, awaiting human check)
- Built: `settings.rs` (shared non-secret config JSON + keys in Credential Manager; old M3 settings files still load), `llm.rs` (chat-completions cleanup call), `prompt.rs` (the EXECUTION_PLAN.md section 7 prompt verbatim - edit this file to tune it - plus the vocabulary addendum), `net.rs` (shared HTTP client and friendly errors). `stt.rs` now sends `language` (default `en`; `auto` omits it) and a `prompt` of "Vocabulary: ..." to Whisper.
- Settings UI: transcription (address/model/language/key), cleanup (address/model/optional key), vocabulary box. Cleanup defaults to Groq `llama-3.3-70b-versatile`; if no separate cleanup key is saved and the address is the same service as transcription, the transcription key is reused.
- UI shows raw transcript (before) and cleaned text (after), plus a "Test cleanup with typed text" box for trying the section 7 examples without speaking.
- Verified: 9 unit tests pass. Not tested against the live LLM (needs the human's key).

### M5 fix - retired Groq model (built, awaiting human re-test)
- `llama-3.3-70b-versatile` returned HTTP 404 (shut down). Default cleanup model is now `openai/gpt-oss-120b` (`settings.rs`). Settings files that still hold the retired id are auto-switched on load (`RETIRED_LLM_MODELS` - add future dead ids there). The cleanup model was already an editable Settings field.
- New `models.rs`: after every Save the app calls the service's `GET {address}/models` with the saved key and warns (never blocks saving) if the transcription or cleanup model is not in the list, suggesting a few valid ones. Fallback models are noted in the Settings hint text.
- Verified: 13 unit tests pass. Live check not run (needs the human's key).

## M4 - Fast-speech robustness (verified by human)
- Built (all in `src-tauri/src/audio.rs`): audio converted to 16 kHz mono (what Whisper uses internally; ~32 KB/s upload) with an averaging low-pass; silence trimmed at the start/end only, keeping 0.35 s padding so fast starts/ends aren't clipped, never touching pauses mid-speech; a 10-minute cap with a clear error message. The whole clip is still one request (no chunking/streaming). UI now shows recorded vs. sent length.
- Verified: 4 unit tests pass (`cargo test` in `src-tauri`) for resampling and trimming. Not tested with real fast speech (needs the human).
- TODO for M5 (requested by human): a vocabulary/prompt hint for names and uncommon words (e.g. "Harshil", "latte") - plan to pass a user-editable word list to Whisper's `prompt` field and to the cleanup LLM's system prompt.
- Test: `npm run tauri dev`, record a long, very fast run-on sentence, compare the transcript.

## M3 - Transcription via user key (verified by human)
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
