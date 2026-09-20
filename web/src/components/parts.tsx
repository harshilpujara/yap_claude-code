import type { ComponentType, ReactNode } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Label } from "@/components/ui/label"

type Icon = ComponentType<{ className?: string; strokeWidth?: number }>

export function PageHeader({ title, description, right }: { title: string; description?: string; right?: ReactNode }) {
  return (
    <div className="flex items-start justify-between gap-4">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-1 text-sm text-muted-foreground">{description}</p>}
      </div>
      {right}
    </div>
  )
}

/** A settings card: icon + title + description, then the fields. */
export function SectionCard({
  icon: Icon, title, description, children,
}: { icon: Icon; title: string; description?: string; children: ReactNode }) {
  return (
    <Card className="[--card-spacing:--spacing(6)]">
      <CardHeader>
        <div className="flex items-center gap-2.5">
          <Icon className="size-[18px] text-muted-foreground" strokeWidth={1.75} />
          <CardTitle>{title}</CardTitle>
        </div>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent className="space-y-5">{children}</CardContent>
    </Card>
  )
}

/** Label + control + optional helper text. */
export function Field({ label, htmlFor, hint, children }: { label: string; htmlFor?: string; hint?: ReactNode; children: ReactNode }) {
  return (
    <div className="space-y-2">
      <Label htmlFor={htmlFor} className="text-sm font-medium">{label}</Label>
      {children}
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  )
}

export function Kbd({ children }: { children: ReactNode }) {
  return (
    <kbd className="rounded-md border bg-card px-1.5 py-0.5 font-sans text-xs font-medium text-foreground shadow-[0_1px_0_var(--border)]">
      {children}
    </kbd>
  )
}
