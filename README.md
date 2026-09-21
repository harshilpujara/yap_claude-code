# yapp

Free voice-to-text for Windows: press a hotkey, talk, and clean text appears at your cursor, in any app.

<!-- TODO: replace with a demo GIF once recorded, e.g. ![yapp demo](docs/demo.gif) -->
> Demo GIF coming soon.

## Features

- **Fast transcription** - your speech is transcribed by the provider you choose, usually in about a second.
- **Filler-word removal** - "um", "uh", stutters and false starts are dropped.
- **AI formatting** - punctuation, capitalization and paragraphs, plus spoken commands like "new line" and "new paragraph". It also works out self-corrections ("Tuesday, no, Wednesday" becomes Wednesday).
- **Auto language detection** - pick a language, or let yapp detect it for each dictation, with a primary language as the fallback for very short clips. See [docs/languages.md](docs/languages.md).
- **No translation** - text is cleaned in the language you spoke, never translated.
- **Voice shortcuts** - say a phrase and get longer text. It only expands when you clearly mean it as a shortcut.
- **Free** - you bring your own API key and pay only your provider for what you use.

**Windows only for now.**

## Download

Get the latest installer (`yapp_<version>_x64-setup.exe`) from the [Releases page](https://github.com/harshilpujara/yap_claude-code/releases). It installs for the current user only (no administrator rights) and adds a Start menu entry. yapp lives in the system tray.

### Windows SmartScreen warning

The installer is **not code-signed yet**, so the first time you run it Windows SmartScreen may show a blue "Windows protected your PC" box. That is expected for a small app without a paid signing certificate; it does not mean the file is harmful. Click **More info**, then **Run anyway**.

## Setup

yapp needs an API key from a speech-to-text and language-model provider. It works with any OpenAI-compatible service, for example [Groq](https://console.groq.com/keys) (the easy start, with a free tier), OpenAI or OpenRouter. Details: [docs/providers.md](docs/providers.md).

1. Get a key from your provider.
2. Open yapp (tray icon, or the Start menu) and go to **AI Services**.
3. Paste the key into the key field and click **Save changes**. Keys are stored in Windows Credential Manager, never in a file.

## Use it

Press **Ctrl+Space** anywhere to start dictating, and press it again to stop. The cleaned text is pasted at your cursor. You can change the hotkey under **Input**.

## Privacy

Audio goes **directly from your PC to the provider you chose, using your own API key**. It never goes to us; there is no yapp server. The transcript goes to your chosen provider for cleanup the same way. Your keys stay on your PC (Windows Credential Manager). Each recording is deleted from disk as soon as it has been sent. yapp keeps no transcript history, has no telemetry or crash reporting, and the Dashboard stores only counts (words, seconds, dictations per day) locally. Check your provider's own privacy policy for what they do with what you send.

## Build from source

Requires [Rust](https://rustup.rs), Node.js 22 and the Visual Studio C++ Build Tools.

```
npm install
npm --prefix web install
npm run tauri dev      # run in development
npm run tauri build    # build the installer: src-tauri/target/release/bundle/nsis/
```

The app window is a React + Tailwind + shadcn/ui project in `web/`; the recording pill is plain HTML/JS in `web/public/`. See [CONTRIBUTING.md](CONTRIBUTING.md) for tests.

## Releasing

Pushing a version tag such as `v0.2.0` builds the installer and creates a **draft** GitHub Release with it attached; review and publish it from the Releases page. Cut one with `npm run release -- X.Y.Z`, then `git push origin main` and `git push origin vX.Y.Z`. The workflow refuses to release if the tag and the app's version disagree.

## Updating

Updating is manual for now: download the newer installer from Releases and run it over the old one. Settings and API keys are kept.

## License

[MIT](LICENSE)
