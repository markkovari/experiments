import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { listEvents, getEventSubscribers } from '../api'

export function EventsPage() {
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
    </div>
  )
}
