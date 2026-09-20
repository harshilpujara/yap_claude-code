import { dateKey, fmtInt } from "@/lib/format"
import { cn } from "@/lib/utils"

const WEEKS = 27
const LEVEL = ["bg-heat-0", "bg-heat-1", "bg-heat-2", "bg-heat-3", "bg-heat-4"]

/** GitHub-style grid: one square per day, darker = more words. Weeks run left to right, Sunday first. */
export function Heatmap({ days }: { days: { date: string; words: number }[] }) {
  const words = new Map(days.map((d) => [d.date, d.words]))
  const today = new Date()
  today.setHours(0, 0, 0, 0)
  const start = new Date(today)
  start.setDate(today.getDate() - today.getDay() - (WEEKS - 1) * 7)
  const max = Math.max(1, ...words.values())

  const cells = Array.from({ length: WEEKS * 7 }, (_, i) => {
    const d = new Date(start)
    d.setDate(start.getDate() + i)
    if (d > today) return { key: i, level: -1, title: "" }
    const w = words.get(dateKey(d)) ?? 0
    const level = w === 0 ? 0 : Math.min(4, Math.max(1, Math.ceil((w / max) * 4)))
    const label = d.toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" })
    return { key: i, level, title: `${label}: ${fmtInt(w)} words` }
  })

  return (
    <div>
      <div className="flex gap-2">
        <div className="grid grid-rows-7 text-[10px] leading-none text-muted-foreground" aria-hidden="true">
          {["", "Mon", "", "Wed", "", "Fri", ""].map((d, i) => (
            <span key={i} className="flex items-center justify-end">{d}</span>
          ))}
        </div>
        <div
          role="img"
          aria-label={`Words dictated per day, last ${WEEKS} weeks`}
          className="grid flex-1 grid-flow-col grid-rows-7 gap-[3px]"
          style={{ gridAutoColumns: "1fr" }}
        >
          {cells.map((c) => (
            <i
              key={c.key}
              title={c.title}
              className={cn("block aspect-square rounded-[3px]", c.level < 0 ? "bg-transparent" : LEVEL[c.level])}
            />
          ))}
        </div>
      </div>
      <div className="mt-3 flex items-center justify-end gap-1 text-[11px] text-muted-foreground" aria-hidden="true">
        <span className="mr-1">Less</span>
        {LEVEL.map((c) => <i key={c} className={cn("block size-[11px] rounded-[3px]", c)} />)}
        <span className="ml-1">More</span>
      </div>
    </div>
  )
}
