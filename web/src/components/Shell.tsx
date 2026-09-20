import type { ReactNode } from "react"
import { ArrowRight, Book, Dashboard, Microphone, Sparks } from "iconoir-react"
import logo from "@/assets/yapp-logo.png"
import { Button } from "@/components/ui/button"
import { Onboarding } from "@/components/Onboarding"
import { PORTFOLIO_URL, api, errorText } from "@/lib/tauri"
import { cn } from "@/lib/utils"
import { useApp, type Page } from "@/state/app-state"

const NAV: { id: Page; label: string; icon: typeof Dashboard }[] = [
  { id: "input", label: "Input", icon: Microphone },
  { id: "services", label: "AI Services", icon: Sparks },
  { id: "vocab", label: "Vocabulary", icon: Book },
]

function NavItem({ id, label, icon: Icon }: { id: Page; label: string; icon: typeof Dashboard }) {
  const { page, setPage } = useApp()
  const active = page === id
  return (
    <button
      type="button"
      onClick={() => setPage(id)}
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
        active ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-accent hover:text-foreground",
      )}
    >
      <Icon className="size-[18px]" strokeWidth={1.75} />
      {label}
    </button>
  )
}

function Credit() {
  const { setStatus } = useApp()
  const open = () => {
    if (PORTFOLIO_URL) api.openUrl(PORTFOLIO_URL).catch((e) => setStatus("Error - " + errorText(e)))
  }
  return (
    <p className="px-3 text-xs text-muted-foreground">
      Created with <span aria-label="love">&hearts;</span> by{" "}
      {PORTFOLIO_URL ? (
        <a href={PORTFOLIO_URL} onClick={(e) => { e.preventDefault(); open() }} className="underline underline-offset-2 hover:text-foreground">
          Harshil
        </a>
      ) : (
        "Harshil"
      )}
    </p>
  )
}

function SaveBar() {
  const { save, status, dirty } = useApp()
  return (
    <div className="sticky bottom-0 border-t bg-background/95 backdrop-blur">
      <div className="mx-auto flex max-w-3xl items-center justify-between gap-4 px-8 py-3">
        <p className="min-w-0 truncate text-sm text-muted-foreground">
          {dirty ? "You have unsaved changes." : status}
        </p>
        <Button size="lg" onClick={() => void save()}>
          Save changes
          <ArrowRight data-icon="inline-end" />
        </Button>
      </div>
    </div>
  )
}

export function Shell({ children }: { children: ReactNode }) {
  const { page } = useApp()
  return (
    <div className="flex h-screen flex-col">
      <header className="flex h-14 flex-none items-center border-b bg-card px-6">
        <img src={logo} alt="yapp" className="h-7 w-auto" />
      </header>
      <div className="flex min-h-0 flex-1">
        <aside className="flex w-56 flex-none flex-col justify-between border-r bg-sidebar p-3">
          <nav aria-label="Sections" className="space-y-1">
            <NavItem id="dashboard" label="Dashboard" icon={Dashboard} />
            <p className="px-3 pt-5 pb-1 text-[11px] font-medium tracking-wider text-muted-foreground uppercase">Settings</p>
            {NAV.map((n) => <NavItem key={n.id} {...n} />)}
          </nav>
          <Credit />
        </aside>
        <div className="flex min-w-0 flex-1 flex-col">
          <main className="min-h-0 flex-1 overflow-y-auto">
            <div className="mx-auto max-w-3xl space-y-6 px-8 py-8">{children}</div>
          </main>
          {page !== "dashboard" && <SaveBar />}
        </div>
      </div>
      <Onboarding />
    </div>
  )
}
