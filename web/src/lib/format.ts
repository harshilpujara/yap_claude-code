export const fmtInt = (n: number) => Math.round(n).toLocaleString()

export function fmtDuration(seconds: number): string {
  const mins = Math.round(seconds / 60)
  if (mins < 1) return "0m"
  if (mins < 60) return `${mins}m`
  return `${Math.floor(mins / 60)}h ${mins % 60}m`
}

const pad2 = (n: number) => String(n).padStart(2, "0")

/** Local date as YYYY-MM-DD (never via UTC, so days match the PC's calendar). */
export const dateKey = (d: Date) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`

// ---- Hotkey capture (browser key event -> "Ctrl+Shift+K" style string) ----
const MODIFIER_CODES = new Set([
  "ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight",
  "AltLeft", "AltRight", "MetaLeft", "MetaRight",
])

function keyName(code: string): string {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3)
  if (/^Digit\d$/.test(code)) return code.slice(5)
  if (code.startsWith("Arrow")) return code.slice(5)
  return code // Space, Enter, F9, ...
}

export function hotkeyFromEvent(e: {
  code: string; ctrlKey: boolean; altKey: boolean; shiftKey: boolean; metaKey: boolean
}): string | null {
  if (MODIFIER_CODES.has(e.code)) return null
  const isFunctionKey = /^F\d{1,2}$/.test(e.code)
  if (!(e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) && !isFunctionKey) return null
  const parts: string[] = []
  if (e.ctrlKey) parts.push("Ctrl")
  if (e.altKey) parts.push("Alt")
  if (e.shiftKey) parts.push("Shift")
  if (e.metaKey) parts.push("Super")
  parts.push(keyName(e.code))
  return parts.join("+")
}
