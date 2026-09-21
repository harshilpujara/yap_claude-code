# Provider setup

yapp talks to any service that offers an **OpenAI-compatible** API, for two jobs:

- **Transcription** (`/audio/transcriptions`, a Whisper-style model)
- **Cleanup** (`/chat/completions`, any chat model)

You set each one separately under **AI Services**: base URL, model name and API key.

## Groq (easy start)

1. Create a key at <https://console.groq.com/keys> (free tier available).
2. In **AI Services**, paste it as the transcription key. The default base URL is `https://api.groq.com/openai/v1`.
3. If the cleanup service is on the same host, yapp reuses that key for cleanup; otherwise add a cleanup key too.

Defaults: `whisper-large-v3-turbo` for transcription and `openai/gpt-oss-120b` for cleanup.

## OpenAI, OpenRouter and others

Set the base URL and model names your provider documents (for example `https://api.openai.com/v1`). Transcription requests use the `verbose_json` response format so yapp can read the detected language and confidence for [Auto-detect](languages.md); the provider must support it.

## Where keys are stored

In Windows Credential Manager, under the service name `yapp`. They are never written to the settings file or the repository.

## Costs and privacy

You pay your provider directly for what you use. Audio and transcripts go straight to your provider with your key, never to yapp's authors.
