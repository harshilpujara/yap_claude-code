import { useState } from "react"
import { Book, Play } from "iconoir-react"
import { Field, PageHeader, SectionCard } from "@/components/parts"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { api, errorText } from "@/lib/tauri"
import { useApp } from "@/state/app-state"

function CleanupTester() {
  const { setStatus } = useApp()
  const [input, setInput] = useState("")
  const [output, setOutput] = useState("")
  const [busy, setBusy] = useState(false)
  const run = async () => {
    setOutput("")
    setBusy(true)
    setStatus("Cleaning up...")
    try {
      setOutput(await api.cleanupText(input))
      setStatus("Done")
    } catch (e) {
      setStatus("Cleanup error - " + errorText(e))
    } finally {
      setBusy(false)
    }
  }
  return (
    <SectionCard icon={Play} title="Test cleanup" description="Try the AI cleanup on typed text - no microphone needed.">
      <Field label="Paste a messy transcript" htmlFor="test-input">
        <Textarea id="test-input" rows={4} spellCheck={false} value={input} onChange={(e) => setInput(e.target.value)} />
      </Field>
      <Button variant="outline" size="lg" disabled={busy || !input.trim()} onClick={() => void run()}>
        Clean this text
      </Button>
      {output && (
        <div>
          <p className="mb-1.5 text-xs font-medium text-muted-foreground">Result</p>
          <p className="rounded-lg border bg-muted/50 p-3 text-sm whitespace-pre-wrap">{output}</p>
        </div>
      )}
    </SectionCard>
  )
}

export function VocabularyPage() {
  const { form, update } = useApp()
  return (
    <>
      <PageHeader title="Vocabulary" description="Help yapp spell your world correctly." />
      <SectionCard icon={Book} title="Names and uncommon words">
        <Field
          label="Comma or new line separated"
          htmlFor="vocabulary"
          hint="Used for both transcription and cleanup so these are spelled correctly."
        >
          <Textarea
            id="vocabulary"
            rows={4}
            spellCheck={false}
            placeholder="Harshil, latte, Kubernetes"
            value={form.vocabulary}
            onChange={(e) => update({ vocabulary: e.target.value })}
          />
        </Field>
      </SectionCard>
      <CleanupTester />
    </>
  )
}
