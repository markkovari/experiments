import { useState, useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { useWorkflowEvents } from './sse'
import { WorkflowsPage } from './pages/WorkflowsPage'
import { RunsPage } from './pages/RunsPage'
import { EventsPage } from './pages/EventsPage'
import { DeployPage } from './pages/DeployPage'
import { SecretsPage } from './pages/SecretsPage'

type Tab = 'runs' | 'workflows' | 'events' | 'deploy' | 'secrets'

function useDarkMode() {
  const [dark, setDark] = useState(() => {
    const stored = localStorage.getItem('theme')
    if (stored) return stored === 'dark'
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  })

  useEffect(() => {
    document.documentElement.classList.toggle('dark', dark)
    localStorage.setItem('theme', dark ? 'dark' : 'light')
  }, [dark])

  return [dark, setDark] as const
}

export function App() {
  const qc = useQueryClient()
  useWorkflowEvents(qc)

  const [tab, setTab] = useState<Tab>('runs')
  const [dark, setDark] = useDarkMode()

  const navItem = (label: string, value: Tab) => (
    <button
      onClick={() => setTab(value)}
      className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
        tab === value
          ? 'border-blue-500 text-blue-600 dark:text-blue-400'
          : 'border-transparent text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100'
      }`}
    >
      {label}
    </button>
  )

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
      <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700 shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center gap-6">
          <span className="font-bold text-lg">⚙ Workflow UI</span>
          <nav className="flex gap-1 flex-1">
            {navItem('Runs', 'runs')}
            {navItem('Workflows', 'workflows')}
            {navItem('Events', 'events')}
            {navItem('Deploy', 'deploy')}
            {navItem('Secrets', 'secrets')}
          </nav>
          <button
            onClick={() => setDark(d => !d)}
            className="p-2 rounded-md text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            title={dark ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {dark ? '☀' : '☾'}
          </button>
        </div>
      </header>

      <main className="max-w-7xl mx-auto">
        {tab === 'runs' && <RunsPage />}
        {tab === 'workflows' && <WorkflowsPage />}
        {tab === 'events' && <EventsPage />}
        {tab === 'deploy' && <DeployPage />}
        {tab === 'secrets' && <SecretsPage />}
      </main>
    </div>
  )
}
