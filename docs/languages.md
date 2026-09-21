# Language settings

Set this under **Input -> Language**.

## Fixed language

Pick one language (English is the default). Transcription is told to expect it, which is the most reliable option if you always speak one language.

## Auto-detect

yapp detects the spoken language for each dictation, and the cleanup step writes the text in **that same language**. It never translates. Mixed-language speech (for example Hindi with English words) is kept as spoken.

### Primary language

Language detection can guess wrong on very short or unclear clips. When Auto-detect is on you also choose a **primary language**. For each dictation:

- If the detected language matches the primary language, or the clip is long and clearly heard, the result is kept.
- If the clip is under about 2 seconds, or transcription confidence is low, and the detected language differs from your primary language, yapp transcribes it again in the primary language and keeps whichever result is more trustworthy (for very short clips, the primary-language one unless it is clearly worse).

Consequence: a very short clip that really is in another language may come out wrong. Say a few more words, or switch Language to that fixed language.

### Diagnosing

Each auto-detect dictation logs the detected language, whether it fell back and why. Run yapp from a terminal to see it.
