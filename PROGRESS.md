# Progress

## Frontend rebuild: shadcn/ui dashboard + settings (built, awaiting human test)
- New frontend in `web/`: Vite + React 19 + TypeScript + Tailwind v4 + shadcn/ui (Radix, style `radix-nova`, components in `web/src/components/ui`). Icons: `iconoir-react` everywhere (shadcn's Select icons were swapped from lucide to iconoir). Font: Inter (variable, bundled offline via `@fontsource-variable/inter`), also in the pill (`web/public/pill.css`, font copied to `web/public/fonts`).
- Tauri now loads `web/dist` (`frontendDist`), with `devUrl` http://localhost:1420, `beforeDevCommand`/`beforeBuildCommand` running the Vite dev server/build. CI runs `npm --prefix web ci` first. The old `src/` (vanilla UI) is gone; the pill files and vendored orbs engine moved unchanged to `web/public/` (pill behavior untouched).
- Design: LIGHT ONLY (no dark variant). Tokens are CSS variables at the top of `web/src/index.css`: neutral greys, #FAFAFA app background, white cards, charcoal text; purple #7C5CFF only for the primary button (Save changes), focus rings, the active sidebar item and the heatmap's intensity steps. No gradients anywhere in the app UI.
- Structure: header with full wordmark; left sidebar Dashboard | Settings: Input, AI Services, Vocabulary; footer credit in the sidebar. Dashboard: status badge + hotkey hint, hero (total words), avg wpm / dictations / time saved, streak + heatmap, Last dictation (when there is one), Privacy card with Reset stats (click twice). Settings pages keep every previous field; one sticky "Save changes" bar saves them all; unsaved edits are never overwritten on window focus; opening from the tray lands on the Dashboard.
- Code map: `web/src/state/app-state.tsx` (settings form, keys, pipeline status, last result), `state/use-stats.ts`, `lib/tauri.ts` (typed commands/events; `PORTFOLIO_URL` lives here and is STILL EMPTY - "Harshil" is plain text until set), `pages/*`, `components/Shell.tsx`, `components/Heatmap.tsx`.
- Gotchas found: the shadcn CLI defaulted to Base UI and imported `cn` from a stray npm package - fixed by hand (Radix style, real `@/lib/utils`, Radix `data-[state=...]` selectors). A stray `postcss.config.js` on the Desktop (outside the repo) broke Vite until `css.postcss` was set explicitly in `vite.config.ts`.
- Verified: `tsc` + `vite build` clean, `cargo check` passes, all four pages rendered headlessly with a stubbed backend. Not yet run inside the real app.

## Pill: thinking-orbs (built, awaiting human test)
- Package: `thinking-orbs` 0.3.1 (MIT), installed with npm. It is a React component, but it ships a framework-free `/engine` export (plain geometry + canvas code). So the pill stays vanilla JS: `dist/engine.es.js` (19 KB, no imports) is vendored to `src/vendor/thinking-orbs-engine.js` with its licence (`npm run vendor:orbs` refreshes it) and loaded as an ES module by `pill.js`. No React, no bundler.
- Real state names (verified in the package typings): working, searching, solving, listening, connecting, weaving, composing, breathing, shaping. Mapping: recording -> `listening` (waveform rolling through rings) with "keep yapping"; processing -> `composing` with "cleaning up your yap..." (was `working`, then `breathing`; changed at the human's request); failed -> no orb, the existing red FAILED pill with the short reason from M10; idle -> hidden.
- Mic reactivity: the package has no amplitude input, so `yapp://level` drives the orb's clock speed (0.6x-2.6x) and swells it (88%-112%) while recording.
- Tint: the package paints grey only, so `pill.js` takes its frames (`MODE_FRAMES`) and paints the dots itself in purple -> pink. It uses the package's 20px preset (chunkier dots); the 64 preset was too faint at pill size.
- Pill tweaks (later): processing now uses the `composing` orb (undulating sash). The capsule hugs its content, 60 px tall, 44 px orb, 16 px text (failed text 14 px). It slides up from the bottom edge of the usable screen area and slides back down on the way out: the transparent window (360x86) sits flush on that edge (`place_pill` in `ui.rs`), the page holds the 24 px gap, and `pill.css` animates `.in` (0.38 s ease-out in, 0.26 s ease-in out). On hide Rust emits `yapp://pill-hide`, waits 380 ms for the slide, then hides the window (a newer show cancels the hide). Reduced-motion users get no animation.
- Unchanged: bottom-centre placement, non-focusable + click-through, show/hide timing (`ui.rs` untouched). The old bar waveform code is gone.
- Verified in a headless browser with simulated events (recording quiet/loud, processing, failed); not yet in the real app.

## Redesign: brand + dashboard-first layout (built, awaiting human test)
- Brand: `brand/yapp-mark.png` -> app icon. It has a lot of empty margin and a white background, so `brand/yapp-icon-source.png` is that mark cropped tight and centred on a white rounded tile (made with System.Drawing); `npx tauri icon brand/yapp-icon-source.png` generated everything in `src-tauri/icons/` (exe, taskbar, window, tray, installer). `app-icon.png` is the same source. The full wordmark `brand/yapp-logo.png` is trimmed and downscaled to `src/assets/yapp-logo.png` (73 KB) for the header; in dark mode it is flipped to white with a CSS filter.
- Layout: top bar with the wordmark; left sidebar Dashboard | Settings: Input, AI Services, Vocabulary; window is now 940x720 (min 720x540). Opening the window from the tray always lands on the Dashboard.
- Dashboard: status indicator + hotkey hint, gradient hero (words), avg wpm / dictations / time saved cards, streak + purple-to-pink heatmap, Last dictation, Privacy card, Reset stats. Insights tab is folded into this (same local stats code).
- Input: hotkey, language, insert method, start on login (applies instantly). AI Services: transcription + cleanup address/model/key, key-saved notes, remove-key buttons, model-check warnings. Vocabulary: word list + the typed-text cleanup tester. A sticky "Save changes" bar appears on the settings pages; every previous field/behavior is kept. Unsaved edits are no longer overwritten when the window regains focus.
- Theme: all colors are variables at the top of `styles.css` (purple #7C5CFF, pink #FF6EC7, gradient, neutrals, button shadows, heatmap levels) with dark mode. Buttons are shadcn-style (inner highlight + soft shadow). Icons are inline lucide-style SVGs. The pill keeps its states; its waveform is now a purple-to-pink gradient and the recording dot is pink (FAILED stays red).
- Footer: "Created with <heart> by Harshil". `PORTFOLIO_URL` (top of `app.js`) is EMPTY until the human supplies the address; until then the name is plain text. When set, it opens in the default browser through the `open_url` command (https only).
- Verified: cargo check + 19 tests pass; all pages previewed headlessly with fake data (dark and light). Not run inside the real app (debug exe was locked by a running yapp).

## Insights tab - local usage stats (built, awaiting human test)
- New "Insights" tab next to "Settings" in the settings window (remembers the last tab). Cards: words dictated (hero), average words/min, dictations; estimated time saved (words at ~40 wpm typing minus audio time spoken); current + longest streak with a 27-week GitHub-style heatmap (5 green levels, relative to your busiest day; hover a square for the day's words). "Reset stats" (click twice) erases everything.
- Storage: `stats.json` in the app config folder, per local day: words, dictations, audio seconds. Counts only, never text. Updated once per successful dictation (after insert succeeds, or when insertion is set to off); failed dictations count for nothing. No network, no accounts. Code: `src-tauri/src/stats.rs` (tested: word count, streak rules, wpm/time saved), hooks in `pipeline.rs`, UI in `index.html`/`app.js`.
- Theme: `src/styles.css` now holds all colors as CSS variables at the top. Default = Palette 1 (sage/forest, cream #F7F6F1, teal-green #1F4E45) with a matching dark variant that follows the system setting; Palettes 2 (mono-green flat) and 3 (dark-first mint with glowing heatmap) are ready as commented blocks. The whole Settings page uses these tokens. The recording pill keeps its own dark style (`pill.css`).
- Notes: WPM uses audio time sent to transcription (includes ~0.35 s padding and pauses), so it reads a little low. Streak days use the PC's local date. Previewed headlessly with fake data in light and dark; not yet run inside the real app. Debug build could not be relinked because yapp.exe was running (cargo check and 19 tests pass).

## M11 - Packaging + release pipeline (built, awaiting first real release)
- Bundler: NSIS installer `yapp_<version>_x64-setup.exe`, product name/publisher "yapp", icon set, per-user install (no admin), English only. Verified locally with `npm run tauri build`.
- Versioning: one number kept in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (+ lockfile). `npm run release -- X.Y.Z` (`scripts/release.mjs`) updates all, commits, and creates tag `vX.Y.Z`; it refuses on a dirty tree or an existing tag and never pushes.
- Workflow: still builds on push to `main`/PRs; a pushed `v*` tag also checks tag == app version and attaches the installer to a GitHub Release (`softprops/action-gh-release`, auto release notes).
- README: "Cut a release" steps, plain-English SmartScreen/unsigned-app note (More info -> Run anyway; signing optional and paid, later), manual update path.
- Not yet verified: the tag-triggered Release run (needs the human to push the first tag).

## M10 - Reliability & privacy defaults (built, awaiting human failure-case testing)
- Checked first: no "keep recent recordings"/audio-retention setting was ever added, so nothing to remove.
- Privacy: the temp WAV is deleted right after transcription finishes (success or failure), on any pipeline exit (`ProcessingGuard`), and at every app start. The "Show recording in folder" button and `reveal_recording` command are gone. No transcript logging (the only `eprintln!` prints microphone error text, never speech), no telemetry, no crash upload. "Last dictation" in Settings is memory-only. Settings has a Privacy section and the README states the same.
- Failures now show a red FAILED pill for ~5.5 s with a short reason (full text in Settings status): no microphone / microphone blocked or unplugged (also mid-recording), no API key, invalid key (401/403), rate limit (429), no internet, timeout, wrong model/address (404), service error (5xx/other), bad response, recording too long, no speech / silent mic, cleanup returned nothing, insert failed. On any failure nothing is inserted. A panic inside the pipeline shows FAILED instead of leaving the app stuck.
- Network: 10 s connect timeout (offline fails fast), 90 s overall timeout per request.
- Code: `pipeline.rs` (`fail`, `short_reason`, `ProcessingGuard`), `audio.rs`, `net.rs`, `pill.*`, Settings page. 17 unit tests pass (new: short-reason mapping, level meter).
- To test on purpose: unplug/disable mic; Windows Settings > Privacy > Microphone off; remove the key; save a wrong key; turn Wi-Fi off; set the model to a bogus name; press the hotkey and stay silent; hammer requests to hit a rate limit (hard to force).

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

### RESOLVED (by removal) - Type (typing-fallback) insertion mode
- Original symptom: Type mode cut dictated text off mid-sentence in Notepad.
- Safe repro attempt (2026-09-20): a probe typed a 178-character sentence into windows created by the test itself (WinForms and WPF text boxes; aborts if the window loses focus). Both the old one-burst typing and the stash's chunked typing produced the exact text 60/60 times. So the bug could not be reproduced anywhere safe; it was only ever seen in Notepad, which cannot be isolated from the user's own session. The chunked-typing stash therefore stays unverified and was NOT adopted (the stash still exists locally and can be dropped).
- Decision: no fix I can prove, so Type mode was removed. Settings offers only Paste and "Don't insert"; a saved "type" setting is read as Paste. `insert.rs` has no typing code and no automatic fallback: if pasting fails the pill shows FAILED ("Couldn't insert the text"), nothing is inserted, and the cleaned text stays under Last dictation in Settings. Revisit only with a way to reproduce in Notepad.

## M7 - Insert text at the cursor (verified by human, Paste mode)
- Human-verified: Paste-mode insertion works in Chrome, Notepad and Slack. Type mode was later removed (see RESOLVED above).
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
