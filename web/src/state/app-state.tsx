// App-wide state: the settings form (shared by every settings page and saved together),
// the live pipeline status, the last dictation, and the one-line status message.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from "react"
import {
  api, errorText, on,
  type Config, type PipelineState, type ResultPayload, type StatePayload,
} from "@/lib/tauri"

export type Page = "dashboard" | "input" | "services" | "vocab" | "shortcuts"

const EMPTY_FORM: Config = {
  stt_base_url: "", stt_model: "", stt_language: "en", primary_language: "en",
  llm_base_url: "", llm_model: "", vocabulary: "", shortcuts: [],
  hotkey: "Ctrl+Space", insert_method: "paste",
}

interface AppState {
  page: Page
  setPage: (p: Page) => void

  form: Config
  update: (patch: Partial<Config>) => void
  sttKey: string
  llmKey: string
  setSttKey: (v: string) => void
  setLlmKey: (v: string) => void
  hasSttKey: boolean
  hasLlmKey: boolean
  /** False until the first read of the saved settings has finished. */
  settingsLoaded: boolean
  dirty: boolean
  save: () => Promise<void>
  removeKey: (which: "stt" | "llm") => Promise<void>
  modelWarnings: string[] | null

  pipeline: PipelineState
  status: string
  setStatus: (s: string) => void
  hotkeyError: string | null
  lastResult: ResultPayload | null
}

const Ctx = createContext<AppState | null>(null)

export function useApp(): AppState {
  const v = useContext(Ctx)
  if (!v) throw new Error("useApp must be used inside <AppStateProvider>")
  return v
}

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [page, setPage] = useState<Page>("dashboard")
  const [form, setForm] = useState<Config>(EMPTY_FORM)
  const [sttKey, setSttKey] = useState("")
  const [llmKey, setLlmKey] = useState("")
  const [hasSttKey, setHasSttKey] = useState(false)
  const [hasLlmKey, setHasLlmKey] = useState(false)
  const [settingsLoaded, setSettingsLoaded] = useState(false)
  const [dirty, setDirty] = useState(false)
  const [modelWarnings, setModelWarnings] = useState<string[] | null>(null)
  const [pipeline, setPipeline] = useState<PipelineState>("idle")
  const [status, setStatus] = useState("")
  const [hotkeyError, setHotkeyError] = useState<string | null>(null)
  const [lastResult, setLastResult] = useState<ResultPayload | null>(null)
  const dirtyRef = useRef(false)
  dirtyRef.current = dirty

  const load = useCallback(async (force: boolean) => {
    if (!force && dirtyRef.current) return // never wipe unsaved edits
    const s = await api.getSettings()
    const { has_stt_key, has_llm_key, ...cfg } = s
    setForm(cfg)
    setHasSttKey(has_stt_key)
    setHasLlmKey(has_llm_key)
    setSttKey("")
    setLlmKey("")
    setDirty(false)
    setSettingsLoaded(true)
  }, [])

  const refreshHotkeyStatus = useCallback(() => {
    api.getHotkeyStatus().then((h) => setHotkeyError(h.error)).catch(() => {})
  }, [])

  useEffect(() => {
    load(true).catch((e) => setStatus("error - " + errorText(e)))
    refreshHotkeyStatus()
    const offState = on<StatePayload>("yapp://state", (p) => {
      setPipeline(p.state)
      setStatus(p.message)
    })
    const offResult = on<ResultPayload>("yapp://result", setLastResult)
    // Opening the window from the tray always lands on the Dashboard.
    const offOpened = on<null>("yapp://opened", () => setPage("dashboard"))
    // The window is hidden most of the time; refresh whenever it comes back to the front.
    const onFocus = () => {
      load(false).catch(() => {})
      refreshHotkeyStatus()
    }
    window.addEventListener("focus", onFocus)
    return () => {
      offState(); offResult(); offOpened()
      window.removeEventListener("focus", onFocus)
    }
  }, [load, refreshHotkeyStatus])

  const update = useCallback((patch: Partial<Config>) => {
    setForm((f) => ({ ...f, ...patch }))
    setDirty(true)
  }, [])

  const persist = useCallback(
    async (stt: string | null, llm: string | null) => {
      try {
        await api.saveSettings(form, stt, llm)
        setDirty(false)
        dirtyRef.current = false
        await load(true)
        refreshHotkeyStatus()
        setStatus("Settings saved - checking models...")
        const check = await api.checkModels()
        setModelWarnings(check.warnings)
        setStatus(check.warnings.length ? "Settings saved - see the model warnings" : "Settings saved and verified")
      } catch (e) {
        setStatus("Error - " + errorText(e))
      }
    },
    [form, load, refreshHotkeyStatus],
  )

  // For each key: null keeps the stored key, "" removes it, text replaces it.
  const save = useCallback(
    () => persist(sttKey.trim() === "" ? null : sttKey.trim(), llmKey.trim() === "" ? null : llmKey.trim()),
    [persist, sttKey, llmKey],
  )
  const removeKey = useCallback(
    (which: "stt" | "llm") => persist(which === "stt" ? "" : null, which === "llm" ? "" : null),
    [persist],
  )

  const value = useMemo<AppState>(
    () => ({
      page, setPage, form, update, sttKey, llmKey, setSttKey: (v) => { setSttKey(v); setDirty(true) },
      setLlmKey: (v) => { setLlmKey(v); setDirty(true) }, hasSttKey, hasLlmKey, settingsLoaded, dirty, save, removeKey,
      modelWarnings, pipeline, status, setStatus, hotkeyError, lastResult,
    }),
    [page, form, update, sttKey, llmKey, hasSttKey, hasLlmKey, settingsLoaded, dirty, save, removeKey, modelWarnings,
      pipeline, status, hotkeyError, lastResult],
  )
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>
}
