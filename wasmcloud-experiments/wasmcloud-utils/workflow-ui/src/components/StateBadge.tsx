const COLOURS: Record<string, string> = {
  running:   'bg-blue-100 text-blue-800',
  succeeded: 'bg-green-100 text-green-800',
  failed:    'bg-red-100 text-red-800',
  cancelled: 'bg-gray-100 text-gray-700',
  pending:   'bg-yellow-100 text-yellow-800',
  skipped:   'bg-purple-100 text-purple-700',
}

export function StateBadge({ state }: { state: string }) {
  const cls = COLOURS[state] ?? 'bg-gray-100 text-gray-600'
  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${cls}`}>
      {state}
    </span>
  )
}
