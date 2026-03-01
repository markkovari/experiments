import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useWorkflowEvents } from './sse'
import { WorkflowsPage } from './pages/WorkflowsPage'
import { RunsPage } from './pages/RunsPage'
import { EventsPage } from './pages/EventsPage'
import { DeployPage } from './pages/DeployPage'

type Tab = 'runs' | 'workflows' | 'events' | 'deploy'

export function App() {
  const qc = useQueryClient()
  useWorkflowEvents(qc)

  const [tab, setTab] = useState<Tab>('runs')

  const navItem = (label: string, value: Tab) => (
    <button
      onClick={() => setTab(value)}
      className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
        tab === value
          ? 'border-blue-600 text-blue-600'
          : 'border-transparent text-gray-600 hover:text-gray-900'
      }`}
    >
      {label}
    </button>
  )

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center gap-6">
          <span className="font-bold text-lg">⚙ Workflow UI</span>
          <nav className="flex gap-1">
            {navItem('Runs', 'runs')}
            {navItem('Workflows', 'workflows')}
            {navItem('Events', 'events')}
            {navItem('Deploy', 'deploy')}
          </nav>
        </div>
      </header>

      <main className="max-w-7xl mx-auto">
        {tab === 'runs' && <RunsPage />}
        {tab === 'workflows' && <WorkflowsPage />}
        {tab === 'events' && <EventsPage />}
        {tab === 'deploy' && <DeployPage />}
      </main>
    </div>
  )
}
