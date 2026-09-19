# Flow

Free voice-to-text for Windows. Hold a hotkey, talk, release, and clean text appears at your cursor.

> Status: early development (M1 - empty app window + automatic Windows installer build).

## Cost
The app is free. If you use cloud transcription/AI cleanup, you pay your own API provider using your own key. The developer pays nothing and ships no keys.

## Privacy
Only the audio clip and its transcript are sent, to the API providers you choose, using your own key. Keys stay on your PC (Windows Credential Manager). No telemetry.

## Develop
Requires Rust, Node.js and the Visual Studio C++ Build Tools.

```
npm install
npm run tauri dev
```

## Build an installer
Pushing to GitHub runs `.github/workflows/build.yml`, which builds the Windows installer and uploads it as a build artifact (Actions tab -> the run -> Artifacts).
