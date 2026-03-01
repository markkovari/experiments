import React, { useState } from 'react'
import { useQuery, useQueries } from '@tanstack/react-query'
import { listWorkflows, listRuns } from '../api'
import type { RunRecord } from '../api'
import { StateBadge } from '../components/StateBadge'
import { RunDetail } from '../components/RunDetail'

const STATES = ['', 'running', 'succeeded', 'failed', 'cancelled']

function fmtMs(ms: number | null | undefined): string {
  if (!ms) return '—'
  return new Date(ms).toLocaleTimeString()
}

export function RunsPage() {
  const [selectedWf, setSelectedWf] = useState<string | null>(null)
  const [stateFilter, setStateFilter] = useState('')
  const [expandedRunId, setExpandedRunId] = useState<string | null>(null)

  const { data: wfData } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => listWorkflows(1, 200),
  })
  const wfNames = wfData?.items ?? []

  // Single-workflow mode
  const { data: singleData, isLoading: singleLoading } = useQuery({
    queryKey: ['runs', selectedWf, stateFilter],
    queryFn: () => listRuns(selectedWf!, stateFilter || undefined, 1, 200),
    enabled: selectedWf !== null && wfNames.length > 0,
    refetchInterval: 5_000,
  })

  // All-workflows fan-out
  const allRunsQueries = useQueries({
    queries: wfNames.map(name => ({
      queryKey: ['runs', name, stateFilter],
      queryFn: () => listRuns(name, stateFilter || undefined, 1, 200),
      enabled: selectedWf === null && wfNames.length > 0,
      refetchInterval: 5_000,
    })),
  })

  const isLoading = selectedWf !== null ? singleLoading : allRunsQueries.some(q => q.isLoading)

  const runs: RunRecord[] = selectedWf !== null
    ? (singleData?.items ?? [])
    : allRunsQueries
        .flatMap(q => q.data?.items ?? [])
        .sort((a, b) => b.created_at_ms - a.created_at_ms)

  return (
    <div className="p-6 space-y-4">
      <h1 className="text-2xl font-bold">Runs</h1>

      <div className="flex gap-4 flex-wrap items-center">
        <select
          className="border rounded px-2 py-1 text-sm"
          value={selectedWf ?? ''}
          onChange={(e) => setSelectedWf(e.target.value || null)}
        >
          <option value="">All workflows</option>
          {wfNames.map((n) => <option key={n} value={n}>{n}</option>)}
        </select>

        <div className="flex gap-1">
          {STATES.map((s) => (
            <button
              key={s}
              onClick={() => setStateFilter(s)}
              className={`px-3 py-1 rounded text-xs border ${stateFilter === s ? 'bg-blue-600 text-white border-blue-600' : 'hover:bg-gray-50'}`}
            >
              {s || 'All'}
            </button>
          ))}
        </div>
      </div>

      {isLoading && <p className="text-gray-500">Loading…</p>}

      <table className="w-full text-sm border rounded overflow-hidden">
        <thead className="bg-gray-50 text-left">
          <tr>
            <th className="px-4 py-2 font-medium w-6"></th>
            <th className="px-4 py-2 font-medium">Run ID</th>
            <th className="px-4 py-2 font-medium">Workflow</th>
            <th className="px-4 py-2 font-medium">State</th>
            <th className="px-4 py-2 font-medium">Created</th>
          </tr>
        </thead>
        <tbody>
          {runs.map((run: RunRecord) => {
            const isExpanded = expandedRunId === run.run_id
            return (
              <React.Fragment key={run.run_id}>
                <tr
                  className="border-t hover:bg-gray-50 cursor-pointer"
                  onClick={() => setExpandedRunId(isExpanded ? null : run.run_id)}
                >
                  <td className="px-4 py-2 text-gray-400 text-xs">{isExpanded ? '▼' : '▶'}</td>
                  <td className="px-4 py-2 font-mono text-xs">{run.run_id}</td>
                  <td className="px-4 py-2 text-gray-600">{run.wf_name}</td>
                  <td className="px-4 py-2"><StateBadge state={run.state} /></td>
                  <td className="px-4 py-2 text-gray-500">{fmtMs(run.created_at_ms)}</td>
                </tr>
                {isExpanded && (
                  <tr>
                    <td colSpan={5} className="p-0">
                      <RunDetail
                        runId={run.run_id}
                        wfName={run.wf_name}
                        state={run.state}
                        createdAtMs={run.created_at_ms}
                      />
                    </td>
                  </tr>
                )}
              </React.Fragment>
            )
          })}
          {!isLoading && !runs.length && (
            <tr>
              <td colSpan={5} className="px-4 py-6 text-center text-gray-400">
                No runs found.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  )
}
