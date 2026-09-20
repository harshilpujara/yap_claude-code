import { useCallback, useEffect, useState } from "react"
import { api, on, type StatsView } from "@/lib/tauri"

/** Local usage stats (counts only). Reloads after every dictation and when the window refocuses. */
export function useStats() {
  const [stats, setStats] = useState<StatsView | null>(null)
  const load = useCallback(() => {
    api.getStats().then(setStats).catch(() => {})
  }, [])
  useEffect(() => {
    load()
    const off = on<null>("yapp://stats", load)
    window.addEventListener("focus", load)
    return () => {
      off()
      window.removeEventListener("focus", load)
    }
  }, [load])
  return { stats, reload: load }
}
