import { useEffect, useState } from "react"
import { KeyCommand, Microphone, Timer } from "iconoir-react"
import { Field, PageHeader, SectionCard } from "@/components/parts"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { hotkeyFromEvent } from "@/lib/format"
import { api, errorText } from "@/lib/tauri"
import { useApp } from "@/state/app-state"

function Autostart() {
  const { setStatus } = useApp()
  const [on, setOn] = useState(false)
  useEffect(() => {
    api.getAutostart().then(setOn).catch(() => {})
  }, [])
  const change = async (next: boolean) => {
    setOn(next)
    try {
      await api.setAutostart(next)
      setStatus(next ? "yapp will start when you sign in" : "Start on login turned off")
    } catch (e) {
      setOn(!next)
      setStatus("Error - " + errorText(e))
    }
  }
  return (
    <div className="flex items-center justify-between gap-4">
      <div>
        <Label htmlFor="autostart" className="text-sm font-medium">Start yapp when I sign in to Windows</Label>
        <p className="mt-1 text-xs text-muted-foreground">Changes here apply right away. Everything else needs Save changes.</p>
      </div>
      <Switch id="autostart" checked={on} onCheckedChange={(v) => void change(v)} />
    </div>
  )
}

export function InputPage() {
  const { form, update } = useApp()
  return (
    <>
      <PageHeader title="Input" description="How you start dictating and where the text goes." />

      <SectionCard icon={KeyCommand} title="Hotkey" description="Press once to start, again to stop.">
        <Field
          label="Toggle recording (click the box, then press the keys you want)"
          htmlFor="hotkey"
          hint="Use at least one of Ctrl / Alt / Shift with a key, or a function key like F9."
        >
          <Input
            id="hotkey"
            readOnly
            spellCheck={false}
            value={form.hotkey}
            className="cursor-pointer text-center font-medium"
            onKeyDown={(e) => {
              e.preventDefault()
              const combo = hotkeyFromEvent(e.nativeEvent)
              if (combo) update({ hotkey: combo })
            }}
          />
        </Field>
        <Button variant="outline" size="lg" onClick={() => update({ hotkey: "Ctrl+Space" })}>
          Reset to Ctrl+Space
        </Button>
      </SectionCard>

      <SectionCard icon={Microphone} title="Language & inserting text">
        <Field label='Language (e.g. en, hi, es - or "auto" to detect)' htmlFor="stt-language">
          <Input id="stt-language" spellCheck={false} value={form.stt_language} onChange={(e) => update({ stt_language: e.target.value })} />
        </Field>
        <Field label="How the cleaned text reaches your cursor" htmlFor="insert-method">
          <Select value={form.insert_method} onValueChange={(v) => update({ insert_method: v })}>
            <SelectTrigger id="insert-method" className="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="paste">Paste (fast; your clipboard is restored afterwards)</SelectItem>
              <SelectItem value="off">{"Don't insert (only show it under Last dictation)"}</SelectItem>
            </SelectContent>
          </Select>
        </Field>
      </SectionCard>

      <SectionCard icon={Timer} title="Startup">
        <Autostart />
      </SectionCard>
    </>
  )
}
