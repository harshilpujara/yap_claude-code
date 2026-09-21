import { Flash, Plus, Trash } from "iconoir-react"
import { PageHeader, SectionCard } from "@/components/parts"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { useApp } from "@/state/app-state"

export function ShortcutsPage() {
  const { form, update } = useApp()
  const rows = form.shortcuts
  const setRow = (i: number, patch: Partial<{ trigger: string; expansion: string }>) =>
    update({ shortcuts: rows.map((r, j) => (j === i ? { ...r, ...patch } : r)) })
  return (
    <>
      <PageHeader title="Shortcuts" description="Say a phrase, get longer text." />
      <SectionCard
        icon={Flash}
        title="Voice shortcuts"
        description="yapp expands a phrase only when you clearly mean it as a shortcut. The same words inside a normal sentence stay as you said them. Click Save to apply."
      >
        {rows.length === 0 && <p className="text-sm text-muted-foreground">No shortcuts yet.</p>}
        <ul className="space-y-3">
          {rows.map((r, i) => (
            <li key={i} className="grid grid-cols-[1fr_1.6fr_auto] items-start gap-3">
              <Input
                aria-label="Trigger phrase"
                placeholder="my email"
                spellCheck={false}
                value={r.trigger}
                onChange={(e) => setRow(i, { trigger: e.target.value })}
              />
              <Textarea
                aria-label="Expansion"
                rows={1}
                placeholder="me@example.com"
                spellCheck={false}
                value={r.expansion}
                onChange={(e) => setRow(i, { expansion: e.target.value })}
              />
              <Button
                variant="ghost"
                size="icon"
                aria-label="Remove shortcut"
                onClick={() => update({ shortcuts: rows.filter((_, j) => j !== i) })}
              >
                <Trash className="size-4" strokeWidth={1.75} />
              </Button>
            </li>
          ))}
        </ul>
        <Button
          variant="outline"
          size="lg"
          onClick={() => update({ shortcuts: [...rows, { trigger: "", expansion: "" }] })}
        >
          <Plus className="size-4" strokeWidth={1.75} />
          Add shortcut
        </Button>
      </SectionCard>
    </>
  )
}
