import { useState } from 'react'
import { useQuery, useQueries, useMutation } from '@tanstack/react-query'
import { listEvents, getEventSubscribers, listWorkflows, getWorkflow, fireTrigger } from '../api'
import type { TriggerDef } from '../api'

type ActiveTab = 'events' | 'triggers'

interface TriggerRow {
  wfName: string
  triggerIndex: number
  trigger: TriggerDef
}

function describeTrigger(t: TriggerDef): string {
  switch (t.kind) {
    case 'event':  return `event "${t.event}"`
    case 'cron':   return `cron "${t.schedule}"`
    case 'http':   return `${t.method ?? 'ANY'} ${t.path}`
    case 'pubsub': return `pubsub "${t.topic}"`
  }
}

function TriggerKindBadge({ kind }: { kind: TriggerDef['kind'] }) {
  const colours: Record<TriggerDef['kind'], string> = {
    event:  'bg-blue-100 text-blue-700',
    cron:   'bg-purple-100 text-purple-700',
    http:   'bg-green-100 text-green-700',
    pubsub: 'bg-orange-100 text-orange-700',
  }
  return (
    <span className={`inline-block px-2 py-0.5 rounded text-xs font-medium ${colours[kind]}`}>
      {kind}
    </span>
  )
}

function TriggersPanel() {
  const [firedRuns, setFiredRuns] = useState<Record<string, string>>({})

  const { data: wfList } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => listWorkflows(1, 200),
  })
  const wfNames = wfList?.items ?? []

  const wfQueries = useQueries({
    queries: wfNames.map((name) => ({
      queryKey: ['workflow', name],
      queryFn: () => getWorkflow(name),
    })),
  })

  const rows: TriggerRow[] = wfQueries.flatMap((q, i) => {
    if (!q.data) return []
    return (q.data.triggers ?? []).map((trigger, triggerIndex) => ({
      wfName: wfNames[i],
      triggerIndex,
      trigger,
    }))
  })

  const fireMutation = useMutation({
    mutationFn: ({ wfName, triggerIndex }: { wfName: string; triggerIndex: number }) =>
      fireTrigger(wfName, triggerIndex),
    onSuccess: (data, { wfName, triggerIndex }) => {
      setFiredRuns((prev) => ({
        ...prev,
        [`${wfName}:${triggerIndex}`]: data.run_id,
      }))
    },
  })

  if (rows.length === 0) {
    return <p className="text-gray-400 text-sm">No triggers defined across any workflow.</p>
  }

  return (
    <div className="border rounded overflow-hidden">
      <table className="w-full text-sm">
        <thead className="bg-gray-50 text-left">
          <tr>
            <th className="px-4 py-2 font-medium text-gray-600">Workflow</th>
            <th className="px-4 py-2 font-medium text-gray-600">Trigger</th>
            <th className="px-4 py-2 font-medium text-gray-600">Action</th>
          </tr>
        </thead>
        <tbody className="divide-y">
          {rows.map(({ wfName, triggerIndex, trigger }) => {
            const key = `${wfName}:${triggerIndex}`
            const runId = firedRuns[key]
            const isFiring = fireMutation.isPending &&
              fireMutation.variables?.wfName === wfName &&
              fireMutation.variables?.triggerIndex === triggerIndex
            return (
              <tr key={key} className="hover:bg-gray-50">
                <td className="px-4 py-2 font-mono">{wfName}</td>
                <td className="px-4 py-2 space-x-2">
                  <TriggerKindBadge kind={trigger.kind} />
                  <span className="text-gray-700">{describeTrigger(trigger)}</span>
                </td>
                <td className="px-4 py-2">
                  <div className="flex items-center gap-3">
                    <button
                      onClick={() => fireMutation.mutate({ wfName, triggerIndex })}
                      disabled={isFiring}
                      className="px-3 py-1 bg-blue-600 text-white rounded text-xs hover:bg-blue-700 disabled:opacity-50"
                    >
                      {isFiring ? 'Firing…' : 'Fire ▶'}
                    </button>
                    {runId && (
                      <span className="text-xs text-green-700 bg-green-50 border border-green-200 px-2 py-0.5 rounded font-mono">
                        {runId}
                      </span>
                    )}
                  </div>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </div>
  )
}

export function EventsPage() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('events')
  const [selected, setSelected] = useState<string | null>(null)

  const { data: events, isLoading } = useQuery({
    queryKey: ['events'],
    queryFn: listEvents,
    refetchInterval: 10_000,
  })

  const { data: subs } = useQuery({
    queryKey: ['event-subs', selected],
    queryFn: () => getEventSubscribers(selected!),
    enabled: selected != null,
  })

  return (
    <div className="p-6 space-y-4">
      <h1 className="text-2xl font-bold">Events</h1>

      <div className="flex gap-2 border-b pb-2">
        {(['events', 'triggers'] as ActiveTab[]).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-1.5 rounded-t text-sm font-medium capitalize ${
              activeTab === tab
                ? 'bg-white border border-b-white -mb-px text-blue-700'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {activeTab === 'events' && (
        <>
          {isLoading && <p className="text-gray-500">Loading…</p>}
          <div className="flex gap-6">
            <div className="w-64 space-y-1">
              {(events ?? []).map((name) => (
                <button
                  key={name}
                  onClick={() => setSelected(name)}
                  className={`w-full text-left px-3 py-2 rounded text-sm ${selected === name ? 'bg-blue-100 text-blue-800' : 'hover:bg-gray-50'}`}
                >
                  {name}
                </button>
              ))}
              {!isLoading && !events?.length && (
                <p className="text-gray-400 text-sm px-3">No events registered.</p>
              )}
            </div>

            {selected && (
              <div className="flex-1 border rounded p-4 space-y-2">
                <h2 className="font-semibold">{selected} — Subscribers</h2>
                {(subs ?? []).length === 0
                  ? <p className="text-gray-400 text-sm">No subscribers.</p>
                  : (subs ?? []).map((s) => (
                    <div key={s} className="font-mono text-sm bg-gray-50 px-3 py-1 rounded">{s}</div>
                  ))
                }
              </div>
            )}
          </div>
        </>
      )}

      {activeTab === 'triggers' && <TriggersPanel />}
    </div>
  )
}
