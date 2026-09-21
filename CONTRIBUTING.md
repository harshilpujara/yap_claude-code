# Contributing

Thanks for helping. Bug reports and small, focused pull requests are welcome.

## Development setup

Requires Windows, [Rust](https://rustup.rs), Node.js 22 and the Visual Studio C++ Build Tools.

```
npm install
npm --prefix web install
npm run tauri dev
```

`npm run tauri dev` also starts the web frontend (Vite) for you.

## Project layout

- `src-tauri/` - the Rust backend: recording, transcription, cleanup, hotkey, tray, settings.
- `web/` - the settings window: React, Tailwind and shadcn/ui (Vite).
- `web/public/` - the recording pill: plain HTML/JS.
- `docs/` - user guides. `docs/dev/PROGRESS.md` is the running development log.

## Run the tests

```
cd src-tauri && cargo test          # Rust unit tests
cd web && npx tsc --noEmit          # TypeScript type check
```

`cargo test live_cleanup -- --ignored --nocapture` runs real dictations through your saved API key; it is optional and uses your provider quota.

## Build an installer

```
npm run tauri build
```

The installer ends up in `src-tauri/target/release/bundle/nsis/`. Every push to `main` also builds one in GitHub Actions (Actions tab -> the run -> Artifacts).

## Releases

Pushing a version tag such as `v0.3.0` builds the installer and creates a **draft** GitHub Release with it attached; review and publish it from the Releases page.

1. Commit everything you want in the release.
2. `npm run release -- X.Y.Z` sets the version in every file, commits it and creates the tag.
3. `git push origin main`, then `git push origin vX.Y.Z`.

The workflow refuses to release if the tag and the app's version disagree.

## Open an issue

Use the [issue templates](https://github.com/harshilpujara/yap_claude-code/issues/new/choose). For a bug, include your Windows version, provider, language setting and what happened. **Never paste an API key**, and remove anything private from logs.
