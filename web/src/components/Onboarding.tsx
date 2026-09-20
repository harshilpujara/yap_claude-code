import { useEffect, useRef, useState, type ComponentType, type ReactNode } from "react"
import { ArrowRight, Cloud, Coins, KeyCommand, OpenNewWindow, ShieldCheck } from "iconoir-react"
import { Button } from "@/components/ui/button"
import {
  Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from "@/components/ui/dialog"
import { GROQ_KEYS_URL, PROVIDER_NOTE } from "@/lib/copy"
import { api, errorText } from "@/lib/tauri"
import { useApp } from "@/state/app-state"

type Icon = ComponentType<{ className?: string; strokeWidth?: number }>

function Point({ icon: Icon, title, children }: { icon: Icon; title: string; children: ReactNode }) {
  return (
    <li className="flex gap-3">
      <span className="mt-0.5 grid size-8 flex-none place-items-center rounded-lg bg-muted text-foreground">
        <Icon className="size-[18px]" strokeWidth={1.75} />
      </span>
      <div className="min-w-0">
        <p className="text-sm font-medium">{title}</p>
        <p className="mt-0.5 text-sm leading-relaxed text-muted-foreground">{children}</p>
      </div>
    </li>
  )
}

/**
 * First-run welcome. Decided once per launch, right after the saved settings are read:
 * it opens only if NO API key is saved (so never once a key exists, and never mid-session
 * if a key is removed later). "Later" hides it until the next start.
 */
export function Onboarding() {
  const { settingsLoaded, hasSttKey, hasLlmKey, setPage, setStatus, form } = useApp()
  const [open, setOpen] = useState(false)
  const decided = useRef(false)
  useEffect(() => {
    if (settingsLoaded && !decided.current) {
      decided.current = true
      setOpen(!hasSttKey && !hasLlmKey)
    }
  }, [settingsLoaded, hasSttKey, hasLlmKey])

  const addKey = () => {
    setPage("services")
    setOpen(false)
  }
  const openConsole = () => {
    api.openUrl(GROQ_KEYS_URL).catch((e) => setStatus("Error - " + errorText(e)))
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="max-w-lg gap-5 p-7 sm:max-w-lg">
        <DialogHeader className="gap-1.5">
          <DialogTitle className="text-xl font-semibold tracking-tight">Welcome to yapp</DialogTitle>
          <DialogDescription className="text-sm leading-relaxed">
            Talk instead of typing. yapp turns your voice into clean text at your cursor, in any app.
          </DialogDescription>
        </DialogHeader>

        <ul className="space-y-4">
          <Point icon={Coins} title="Free, with your own key">
            yapp is free. You bring your own API key and only pay your provider for what you use. yapp itself sends
            nothing anywhere except your chosen provider.
          </Point>
          <Point icon={Cloud} title="Pick any provider">
            {PROVIDER_NOTE} <span className="font-medium text-foreground">Groq</span> is the easy start: fast, with a free tier.
          </Point>
          <Point icon={ShieldCheck} title="Private by design">
            Your audio and transcript go only to the provider you choose, on your key. Nothing else leaves your PC.
            No account, no login.
          </Point>
          <Point icon={KeyCommand} title="Dictate anywhere">
            Press <span className="font-medium text-foreground">{form.hotkey}</span> anywhere to start dictating, and
            again to stop. You can rebind it in Settings.
          </Point>
        </ul>

        <p className="text-sm text-muted-foreground">
          How to get a key:{" "}
          <a
            href={GROQ_KEYS_URL}
            onClick={(e) => { e.preventDefault(); openConsole() }}
            className="inline-flex items-center gap-1 font-medium text-primary underline-offset-2 hover:underline"
          >
            console.groq.com/keys
            <OpenNewWindow className="size-3.5" strokeWidth={1.75} />
          </a>{" "}
          &mdash; sign in, create a key, and copy it.
        </p>

        <DialogFooter className="gap-2 sm:justify-end">
          <Button variant="outline" size="lg" onClick={() => setOpen(false)}>Later</Button>
          <Button size="lg" onClick={addKey}>
            Add your API key
            <ArrowRight data-icon="inline-end" />
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
