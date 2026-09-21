<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" alt="yapp icon" width="96" height="96">
</p>

<h1 align="center">yapp</h1>

<p align="center"><b>Talk instead of typing. Free voice-to-text for Windows.</b></p>

<p align="center">
  <a href="https://github.com/harshilpujara/yap_claude-code/releases/latest"><img src="https://img.shields.io/badge/Download%20for%20Windows-7C5CFF?style=for-the-badge&logo=windows&logoColor=white" alt="Download yapp for Windows"></a>
</p>

<!-- TODO: REPLACE THIS PLACEHOLDER with a demo GIF (record one, save as docs/demo.gif, then use: ![yapp demo](docs/demo.gif)) -->
<p align="center"><b>[ DEMO GIF GOES HERE - replace me ]</b></p>

## Why yapp

- **Free and open source.** No subscription, no account, no trial.
- **Bring your own API key.** You pay your speech provider directly for what you use, which is typically very little.
- **Private.** Your audio goes straight from your PC to the provider you choose. It never goes to us.
- **Works in many languages** and never translates you. Speak Hindi, get Hindi. Speak Spanish, get Spanish.

## Quick start

1. **Download** the installer from the [latest release](https://github.com/harshilpujara/yap_claude-code/releases/latest) (the file ending in `-setup.exe`).
2. **Run it.** If Windows warns you, see [Windows says it's unsafe?](#windows-says-its-unsafe) below.
3. **Paste your API key.** Open yapp from the system tray or Start menu, go to **AI Services**, paste your key and click **Save changes**. Don't have one? [Get a free Groq key](https://console.groq.com/keys) in a minute.
4. **Press Ctrl+Space** anywhere, talk, and press it again to stop. Your words appear at your cursor.

## Windows says it's unsafe?

When you run the installer, Windows may show a blue box saying **"Windows protected your PC"**. This happens because the installer isn't code-signed yet. Code-signing certificates cost money every year, and yapp is a free project. It doesn't mean the file is harmful, and you can read all of the source code in this repo.

To continue:

1. Click **More info**.
2. Click **Run anyway**.

## Features

- **Fast transcription** - speech turns into text in about a second.
- **Filler-word removal** - "um", "uh", stutters and false starts disappear.
- **Smart formatting** - punctuation, capital letters and paragraphs, plus spoken commands like "new line" and "new paragraph". Change your mind mid-sentence ("Tuesday, no, Wednesday") and yapp writes Wednesday.
- **Auto language detection** - pick a language, or let yapp work out which one you are speaking each time. Set a main language for very short clips. See [docs/languages.md](docs/languages.md).
- **No translation** - your words stay in the language you spoke.
- **Voice shortcuts** - say "sign off" and get "Regards, Harshil". It only expands when you clearly mean it as a shortcut, not when you use the same words in a normal sentence.
- **Works in any app** - the text is pasted wherever your cursor is.

**Windows only for now.**

## FAQ

**What does it cost me?**
yapp is free. You use your own API key and pay your provider for what you use. Groq has a free tier that is enough for many people. yapp has no subscription and no hidden fees.

**Where does my audio go?**
Directly from your PC to the provider you chose, using your own key. It never goes to us, because there is no yapp server. Each recording is deleted from your PC as soon as it has been sent. yapp keeps no history of what you said, and has no tracking or crash reporting. Your key is stored in Windows Credential Manager. Your provider has its own privacy policy for what they do with what you send.

**Which providers work?**
Any service that copies OpenAI's API, including Groq, OpenAI and OpenRouter. Groq is the easiest start. See [docs/providers.md](docs/providers.md).

**Which languages work?**
Many, including English, Hindi, Gujarati, Spanish, French, German, Portuguese, Japanese, Korean and Chinese. The exact list depends on your provider's speech model. See [docs/languages.md](docs/languages.md).

**Does it work offline?**
No. Transcription and cleanup are done by your provider over the internet, so you need a connection.

**How do I update it?**
Download the newer installer from [Releases](https://github.com/harshilpujara/yap_claude-code/releases) and run it over the old one. Your settings and key are kept.

## License

[MIT](LICENSE). Want to help? See [CONTRIBUTING.md](CONTRIBUTING.md).

<details>
<summary><b>Build from source</b></summary>

<br>

You need [Rust](https://rustup.rs), Node.js 22 and the Visual Studio C++ Build Tools.

```
npm install
npm --prefix web install
npm run tauri dev      # run in development
npm run tauri build    # build the installer: src-tauri/target/release/bundle/nsis/
```

More detail, tests and how releases work: [CONTRIBUTING.md](CONTRIBUTING.md).

</details>
