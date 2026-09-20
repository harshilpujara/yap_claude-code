import { useEffect, useRef, useState, type ComponentType } from "react"
import { Clock, DashboardSpeed, FireFlame, Page, Refresh, ShieldCheck, SoundHigh } from "iconoir-react"
import { Heatmap } from "@/components/Heatmap"
import { Kbd, PageHeader } from "@/components/parts"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { fmtDuration, fmtInt } from "@/lib/format"
import { api, errorText, type PipelineState } from "@/lib/tauri"
import { cn } from "@/lib/utils"
import { useApp } from "@/state/app-state"
import { useStats } from "@/state/use-stats"

const STATE_LABEL: Record<PipelineState, string> = {
  idle: "Idle", recording: "Recording", processing: "Working", error: "Problem",
}
const STATE_DOT: Record<PipelineState, string> = {
  idle: "bg-emerald-500", recording: "bg-red-500 animate-pulse", processing: "bg-amber-500 animate-pulse", error: "bg-red-500",
}

type Icon = ComponentType<{ className?: string; strokeWidth?: number }>

function StatCard({ icon: Icon, label, value, note }: { icon: Icon; label: string; value: string; note?: string }) {
  return (
    <Card className="[--card-spacing:--spacing(5)]">
      <CardHeader>
        <div className="flex items-center gap-2 text-muted-foreground">
          <Icon className="size-4" strokeWidth={1.75} />
          <span className="text-xs font-medium">{label}</span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="text-3xl font-semibold tracking-tight tabular-nums">{value}</div>
        {note && <p className="mt-1 text-xs text-muted-foreground">{note}</p>}
      </CardContent>
    </Card>
  )
}

function ResetStats() {
  const { setStatus } = useApp()
  const [armed, setArmed] = useState(false)
  const timer = useRef<number | undefined>(undefined)
  useEffect(() => () => window.clearTimeout(timer.current), [])
  const click = () => {
    if (!armed) {
      setArmed(true)
      timer.current = window.setTimeout(() => setArmed(false), 4000)
      return
    }
    setArmed(false)
    api.resetStats().catch((e) => setStatus("Error - " + errorText(e)))
  }
  return (
    <Button variant="outline" size="lg" onClick={click} className={cn(armed && "border-destructive/40 text-destructive")}>
      <Refresh data-icon="inline-start" />
      {armed ? "Click again to erase all stats" : "Reset stats"}
    </Button>
  )
}

function CardTitleRow({ icon: Icon, children }: { icon: Icon; children: string }) {
  return (
    <div className="flex items-center gap-2.5">
      <Icon className="size-[18px] text-muted-foreground" strokeWidth={1.75} />
      <CardTitle>{children}</CardTitle>
    </div>
  )
}

export function Dashboard() {
  const { pipeline, status, form, hotkeyError, lastResult } = useApp()
  const { stats } = useStats()
  const empty = !stats || stats.dictations === 0

  return (
    <>
      <PageHeader
        title="Dashboard"
        description="Your dictation at a glance."
        right={
          <Badge variant="outline" className="h-7 gap-2 bg-card px-3 text-xs font-medium">
            <span className={cn("size-2 rounded-full", STATE_DOT[pipeline])} />
            {STATE_LABEL[pipeline]}
          </Badge>
        }
      />

      <div className="-mt-3 space-y-2">
        <p className="text-sm text-muted-foreground">
          Press <Kbd>{form.hotkey}</Kbd> anywhere to start recording, and again to stop.
        </p>
        {status && <p className="text-xs text-muted-foreground">{status}</p>}
        {hotkeyError && (
          <Alert variant="destructive"><AlertDescription>{hotkeyError}</AlertDescription></Alert>
        )}
      </div>

      {/* Hero */}
      <Card className="[--card-spacing:--spacing(8)]">
        <CardContent>
          <p className="text-sm font-medium text-muted-foreground">Words dictated</p>
          <div className="mt-2 text-7xl font-semibold tracking-tighter tabular-nums">
            {fmtInt(stats?.total_words ?? 0)}
          </div>
          <p className="mt-3 text-sm text-muted-foreground">
            {empty ? "Nothing here yet. Dictate something and your stats will show up." : "Since you started using yapp."}
          </p>
        </CardContent>
      </Card>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4">
        <StatCard icon={DashboardSpeed} label="Avg words / min" value={stats?.avg_wpm == null ? "—" : fmtInt(stats.avg_wpm)} />
        <StatCard icon={SoundHigh} label="Dictations" value={fmtInt(stats?.dictations ?? 0)} />
        <StatCard
          icon={Clock}
          label="Time saved"
          value={empty ? "—" : fmtDuration(stats?.time_saved_seconds ?? 0)}
          note={`vs. typing at ~${stats?.typing_wpm ?? 40} wpm`}
        />
      </div>

      {/* Streak */}
      <Card className="[--card-spacing:--spacing(6)]">
        <CardHeader>
          <CardTitleRow icon={FireFlame}>Streak</CardTitleRow>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="flex gap-12">
            <div>
              <div className="text-3xl font-semibold tracking-tight tabular-nums">{stats?.current_streak ?? 0}</div>
              <p className="mt-1 text-xs text-muted-foreground">Current streak (days)</p>
            </div>
            <div>
              <div className="text-3xl font-semibold tracking-tight tabular-nums">{stats?.longest_streak ?? 0}</div>
              <p className="mt-1 text-xs text-muted-foreground">Longest streak (days)</p>
            </div>
          </div>
          <Heatmap days={stats?.days ?? []} />
        </CardContent>
      </Card>

      {/* Last dictation */}
      {lastResult && (
        <Card className="[--card-spacing:--spacing(6)]">
          <CardHeader>
            <CardTitleRow icon={Page}>Last dictation</CardTitleRow>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <p className="mb-1.5 text-xs font-medium text-muted-foreground">Raw transcript (before)</p>
              <p className="rounded-lg border bg-muted/50 p-3 text-sm whitespace-pre-wrap">{lastResult.raw || "(no speech detected)"}</p>
            </div>
            <div>
              <p className="mb-1.5 text-xs font-medium text-muted-foreground">Cleaned (after)</p>
              <p className="rounded-lg border bg-muted/50 p-3 text-sm whitespace-pre-wrap">
                {lastResult.cleanup_error ? "(cleanup failed - see the status above)" : lastResult.clean}
              </p>
            </div>
            <p className="text-xs text-muted-foreground">
              {lastResult.info} Kept in memory only - it disappears when you quit yapp.
            </p>
          </CardContent>
        </Card>
      )}

      {/* Privacy */}
      <Card className="[--card-spacing:--spacing(6)]">
        <CardHeader>
          <CardTitleRow icon={ShieldCheck}>Privacy</CardTitleRow>
        </CardHeader>
        <CardContent className="space-y-3 text-sm leading-relaxed text-muted-foreground">
          <p>
            <span className="font-medium text-foreground">What leaves this PC:</span> only the audio of each recording and its
            transcript, sent to the transcription and cleanup services you chose, using your own keys. Nothing else is sent
            anywhere - no analytics, no telemetry, no crash reports.
          </p>
          <p>
            <span className="font-medium text-foreground">What stays here:</span> your keys (Windows Credential Manager) and your
            settings. Each recording is deleted from disk the moment it has been sent. yapp keeps no history and writes no
            transcripts to any log. These stats are only counts (words, seconds and dictations per day) in a small file on this PC.
          </p>
          <div className="flex justify-end pt-1"><ResetStats /></div>
        </CardContent>
      </Card>
    </>
  )
}
