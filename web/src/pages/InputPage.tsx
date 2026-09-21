import { useEffect, useState } from "react"
import { KeyCommand, Microphone, Timer } from "iconoir-react"
import { Field, PageHeader, SectionCard } from "@/components/parts"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { hotkeyFromEvent } from "@/lib/format"
import { languageOptions } from "@/lib/languages"
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
        <Field
          label="Language"
          htmlFor="stt-language"
          hint="Auto-detect works out the spoken language for each dictation and cleans the text in that same language. It never translates."
        >
          <Select value={form.stt_language} onValueChange={(v) => update({ stt_language: v })}>
            <SelectTrigger id="stt-language" className="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="auto">Auto-detect</SelectItem>
              {languageOptions(form.stt_language === "auto" ? "" : form.stt_language).map((l) => (
                <SelectItem key={l.code} value={l.code}>{l.name}</SelectItem>
              ))}
            </SelectContent>
          </Select>
        </Field>
        {form.stt_language === "auto" && (
          <Field
            label="Primary language"
            htmlFor="primary-language"
            hint="Detection can guess wrong on very short or unclear clips. For those, yapp falls back to this language. Longer, clear clips are still auto-detected."
          >
            <Select value={form.primary_language} onValueChange={(v) => update({ primary_language: v })}>
              <SelectTrigger id="primary-language" className="w-full"><SelectValue /></SelectTrigger>
              <SelectContent>
                {languageOptions(form.primary_language).map((l) => (
                  <SelectItem key={l.code} value={l.code}>{l.name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          </Field>
        )}
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
