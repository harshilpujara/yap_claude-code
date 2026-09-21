// Typed wrappers around the Rust commands and events (see src-tauri/src).
import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"

/** Where "Harshil" in the footer links to (opens in the default browser). */
export const PORTFOLIO_URL = "https://kaliyugg.framer.website/"

export interface Shortcut {
  trigger: string
  expansion: string
}

export interface Config {
  stt_base_url: string
  stt_model: string
  stt_language: string
  llm_base_url: string
  llm_model: string
  vocabulary: string
  shortcuts: Shortcut[]
  hotkey: string
  insert_method: string
}

export interface SettingsView extends Config {
  has_stt_key: boolean
  has_llm_key: boolean
}

export interface StatsView {
  total_words: number
  dictations: number
  avg_wpm: number | null
  time_saved_seconds: number
  current_streak: number
  longest_streak: number
  typing_wpm: number
  days: { date: string; words: number }[]
}

export type PipelineState = "idle" | "recording" | "processing" | "error"

export interface StatePayload {
  state: PipelineState
  message: string
  short?: string | null
}

export interface ResultPayload {
  raw: string
  clean: string
  info: string
  cleanup_error?: string | null
}

export const api = {
  getSettings: () => invoke<SettingsView>("get_settings"),
  saveSettings: (config: Config, sttKey: string | null, llmKey: string | null) =>
    invoke<void>("save_settings", { config, sttKey, llmKey }),
  checkModels: () => invoke<{ warnings: string[] }>("check_models"),
  getHotkeyStatus: () => invoke<{ error: string | null }>("get_hotkey_status"),
  getAutostart: () => invoke<boolean>("get_autostart"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  getOnboardingSeen: () => invoke<boolean>("get_onboarding_seen"),
  setOnboardingSeen: () => invoke<void>("set_onboarding_seen"),
  logOnboardingDecision: (show: boolean, reason: string) =>
    invoke<void>("log_onboarding_decision", { show, reason }),
  getStats: () => invoke<StatsView>("get_stats"),
  resetStats: () => invoke<void>("reset_stats"),
  cleanupText: (raw: string) => invoke<string>("cleanup_text", { raw }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
}

/** Subscribes to a backend event; returns the cleanup function. */
export function on<T>(event: string, handler: (payload: T) => void): () => void {
  let off: UnlistenFn | undefined
  let cancelled = false
  listen<T>(event, (e) => handler(e.payload)).then((fn) => {
    if (cancelled) fn()
    else off = fn
  })
  return () => {
    cancelled = true
    off?.()
  }
}

export const errorText = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e))
