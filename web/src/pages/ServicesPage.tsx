import { Sparks, SoundHigh } from "iconoir-react"
import { Field, PageHeader, SectionCard } from "@/components/parts"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { useApp } from "@/state/app-state"

export function ServicesPage() {
  const {
    form, update, sttKey, llmKey, setSttKey, setLlmKey, hasSttKey, hasLlmKey, removeKey, modelWarnings,
  } = useApp()
  return (
    <>
      <PageHeader title="AI Services" description="yapp uses your own API keys. Any OpenAI-compatible service works." />

      <SectionCard icon={SoundHigh} title="Transcription (speech to text)">
        <Field label="Service address" htmlFor="stt-base-url">
          <Input id="stt-base-url" spellCheck={false} value={form.stt_base_url} onChange={(e) => update({ stt_base_url: e.target.value })} />
        </Field>
        <Field label="Model" htmlFor="stt-model">
          <Input id="stt-model" spellCheck={false} value={form.stt_model} onChange={(e) => update({ stt_model: e.target.value })} />
        </Field>
        <Field
          label="API key"
          htmlFor="stt-key"
          hint={hasSttKey ? "A transcription key is saved." : "No transcription key saved yet."}
        >
          <Input id="stt-key" type="password" autoComplete="off" spellCheck={false} value={sttKey} onChange={(e) => setSttKey(e.target.value)} />
        </Field>
        <Button variant="outline" size="lg" onClick={() => void removeKey("stt")}>Remove transcription key</Button>
      </SectionCard>

      <SectionCard icon={Sparks} title="Cleanup (AI rewrite)">
        <Field label="Service address" htmlFor="llm-base-url">
          <Input id="llm-base-url" spellCheck={false} value={form.llm_base_url} onChange={(e) => update({ llm_base_url: e.target.value })} />
        </Field>
        <Field label="Model" htmlFor="llm-model">
          <Input id="llm-model" spellCheck={false} value={form.llm_model} onChange={(e) => update({ llm_model: e.target.value })} />
        </Field>
        <Field
          label="API key (optional if it is the same service as above)"
          htmlFor="llm-key"
          hint={
            hasLlmKey
              ? "A separate cleanup key is saved."
              : "No separate cleanup key. The transcription key is used if the service is the same."
          }
        >
          <Input id="llm-key" type="password" autoComplete="off" spellCheck={false} value={llmKey} onChange={(e) => setLlmKey(e.target.value)} />
        </Field>
        <Button variant="outline" size="lg" onClick={() => void removeKey("llm")}>Remove cleanup key</Button>
      </SectionCard>

      {modelWarnings &&
        (modelWarnings.length > 0 ? (
          <Alert variant="destructive">
            <AlertTitle>Model check</AlertTitle>
            <AlertDescription className="whitespace-pre-wrap">{modelWarnings.join("\n\n")}</AlertDescription>
          </Alert>
        ) : (
          <Alert>
            <AlertDescription>Both models were found on the service. All good.</AlertDescription>
          </Alert>
        ))}

      <div className="space-y-2 text-xs text-muted-foreground">
        <p>If a model stops working, type a different name in its Model box and Save. Cleanup fallbacks: meta-llama/llama-4-scout-17b-16e-instruct or openai/gpt-oss-20b.</p>
        <p>Keys are stored in Windows Credential Manager on this PC, never in a file. Leave a key box empty to keep the saved key.</p>
        <p>{"What leaves this PC: only each recording's audio and transcript, sent to these two services with your keys. See Privacy on the Dashboard."}</p>
      </div>
    </>
  )
}
