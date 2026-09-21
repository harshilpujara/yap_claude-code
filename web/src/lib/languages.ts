/** Languages offered in the pickers (ISO-639-1 codes, as the transcription service expects). */
export const LANGUAGES: { code: string; name: string }[] = [
  { code: "en", name: "English" }, { code: "hi", name: "Hindi" }, { code: "gu", name: "Gujarati" },
  { code: "mr", name: "Marathi" }, { code: "bn", name: "Bengali" }, { code: "ta", name: "Tamil" },
  { code: "te", name: "Telugu" }, { code: "ur", name: "Urdu" }, { code: "pa", name: "Punjabi" },
  { code: "es", name: "Spanish" }, { code: "fr", name: "French" }, { code: "de", name: "German" },
  { code: "pt", name: "Portuguese" }, { code: "it", name: "Italian" }, { code: "nl", name: "Dutch" },
  { code: "ru", name: "Russian" }, { code: "uk", name: "Ukrainian" }, { code: "pl", name: "Polish" },
  { code: "tr", name: "Turkish" }, { code: "ar", name: "Arabic" }, { code: "he", name: "Hebrew" },
  { code: "ja", name: "Japanese" }, { code: "ko", name: "Korean" }, { code: "zh", name: "Chinese" },
  { code: "id", name: "Indonesian" }, { code: "vi", name: "Vietnamese" }, { code: "th", name: "Thai" },
  { code: "sv", name: "Swedish" }, { code: "el", name: "Greek" },
]

/** The list plus, if a saved code is not in it, that code itself so it still displays. */
export function languageOptions(current: string) {
  return LANGUAGES.some((l) => l.code === current) || !current
    ? LANGUAGES
    : [...LANGUAGES, { code: current, name: current }]
}
