import { AppStateProvider, useApp } from "@/state/app-state"
import { Shell } from "@/components/Shell"
import { Dashboard } from "@/pages/Dashboard"
import { InputPage } from "@/pages/InputPage"
import { ServicesPage } from "@/pages/ServicesPage"
import { VocabularyPage } from "@/pages/VocabularyPage"

function CurrentPage() {
  const { page } = useApp()
  switch (page) {
    case "input": return <InputPage />
    case "services": return <ServicesPage />
    case "vocab": return <VocabularyPage />
    default: return <Dashboard />
  }
}

export default function App() {
  return (
    <AppStateProvider>
      <Shell>
        <CurrentPage />
      </Shell>
    </AppStateProvider>
  )
}
