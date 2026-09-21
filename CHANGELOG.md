# Changelog

## 0.3.0 - unreleased

- **Voice shortcuts:** define trigger phrases and their expansions on the new Shortcuts page (stored locally). The cleanup step expands a phrase only when it is clearly meant as a shortcut; the same words inside a normal sentence stay as spoken.
- **Auto language detection:** Language now offers Auto-detect next to a fixed-language picker (English included). A primary language biases the outcome: very short or low-confidence clips that disagree with it are retried in it, while clear clips keep the detected language.
- **No translation:** cleanup always writes in the language you spoke and keeps mixed-language speech as spoken.
- **Onboarding fix:** the first-run dialog is now decided on mount from the onboarding_seen flag alone, retries a failed check, and logs whether it showed or skipped and why.
- **Hardening:** request timeouts scale with recording size, one retry after a dropped connection, clearer network errors, and a 24 MB upload guard. The hotkey ignores stuck-key repeats and is re-registered after sleep/resume, and the recording pill is recreated if it dies.
- **Icons:** new app icon in every size and a matching installer icon.
- Open-source docs added: README, MIT license, contributing guide, issue templates and provider and language guides.

## 0.2.0 - 2026-09-20

- Renamed the app to yapp; it runs from the system tray.
- New recording pill with the thinking-orbs animation that slides in from the screen edge.
- Local stats and a Dashboard (words, dictations, streaks) that never store your text.
- Settings and Dashboard rebuilt with React, Tailwind and shadcn/ui.
- First-run welcome dialog and a note on which providers work.
- Insert method "type" removed; text is pasted.
- Failure and privacy handling: clearer errors, recordings deleted right after sending.
- New app icon.

0.1.0 was the first release.
