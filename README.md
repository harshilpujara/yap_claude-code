# yapp

Free voice-to-text for Windows. Hold a hotkey, talk, release, and clean text appears at your cursor.

> Status: early development.

## Cost
The app is free. If you use cloud transcription/AI cleanup, you pay your own API provider using your own key. The developer pays nothing and ships no keys.

## Privacy
Only the audio clip and its transcript are sent, to the API providers you choose, using your own key. Keys stay on your PC (Windows Credential Manager). Each recording is deleted from disk as soon as it has been sent; no transcript history or logs are kept; no telemetry or crash reporting. The Insights tab stores only counts (words, seconds, dictations per day) locally, never your text.

## Develop
Requires Rust, Node.js and the Visual Studio C++ Build Tools.

```
npm install
npm --prefix web install
npm run tauri dev
```

The app window is a React + Tailwind + shadcn/ui project in `web/` (Vite). `npm run tauri dev` starts it automatically. The recording pill is plain HTML/JS in `web/public/`.

## Build an installer
Every push to `main` runs `.github/workflows/build.yml`, which builds the Windows installer (`yapp_<version>_x64-setup.exe`) and uploads it as a build artifact (Actions tab -> the run -> Artifacts).

## Cut a release
Pushing a version tag builds the installer and attaches it to a GitHub Release.

1. Make sure everything you want is committed.
2. Run `npm run release -- 0.1.0` (use the new version number). This sets the version in every file, commits it, and creates the tag `v0.1.0`.
3. Run `git push origin main` and then `git push origin v0.1.0`.
4. Wait for the "Build Windows app" run on the Actions tab to go green, then open the repository's Releases page and download the installer.

The workflow refuses to release if the tag and the app's version number disagree.

## Installing, and the Windows "protected your PC" warning
yapp installs for the current user only (no administrator rights needed) and shows up in the Start menu. It runs in the system tray.

The installer is not code-signed, so the first time you run it Windows SmartScreen shows a blue "Windows protected your PC" box. That is normal for small apps without a paid signing certificate; it does not mean the file is harmful. To continue, click **More info**, then **Run anyway**.

Signing is an optional paid step we can add later: a code-signing certificate costs money each year and the signing key would be stored as a GitHub Actions secret. Even a signed app can still show the warning for a while until Windows sees enough people install it.

## Updating
Updating is manual for now: download the newer installer from Releases and run it over the old one. Your settings and API keys are kept.
