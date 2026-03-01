import React, { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listSteps, cancelRun } from '../api'
import type { StepListItem } from '../api'
import { StateBadge } from './StateBadge'
import { StepDetail } from './StepDetail'

function fmtMs(ms: number | null | undefined): string {
  if (!ms) return '—'
  return new Date(ms).toLocaleTimeString()
}

interface Props {
  runId: string
  wfName: string
  state: string
  createdAtMs: number
}

export function RunDetail({ runId, wfName, state, createdAtMs }: Props) {
  const qc = useQueryClient()
  const [expandedStep, setExpandedStep] = useState<string | null>(null)

  const { data, isLoading } = useQuery({
    queryKey: ['steps', runId],
    queryFn: () => listSteps(runId),
    refetchInterval: 5_000,
  })

  const cancel = useMutation({
    mutationFn: () => cancelRun(runId),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['runs'] }),
  })

  return (
    <div className="bg-blue-50 border-t border-blue-200 px-6 py-4 space-y-4">
      <div className="flex flex-wrap items-center gap-4 text-sm">
        <span><span className="text-gray-500">Run:</span> <span className="font-mono text-xs">{runId}</span></span>
        <span><span className="text-gray-500">Workflow:</span> {wfName}</span>
        <span><span className="text-gray-500">State:</span> <StateBadge state={state} /></span>
        <span><span className="text-gray-500">Created:</span> {fmtMs(createdAtMs)}</span>
        {state === 'running' && (
          <button
            onClick={() => cancel.mutate()}
            disabled={cancel.isPending}
            className="px-3 py-1 bg-red-100 text-red-800 rounded text-xs hover:bg-red-200 disabled:opacity-50"
          >
            Cancel Run
          </button>
        )}
      </div>

      {isLoading && <p className="text-xs text-gray-400">Loading steps…</p>}

      <table className="w-full text-sm border rounded overflow-hidden bg-white">
        <thead className="bg-gray-50 text-left">
          <tr>
            <th className="px-4 py-2 font-medium w-6"></th>
            <th className="px-4 py-2 font-medium">Step</th>
            <th className="px-4 py-2 font-medium">State</th>
            <th className="px-4 py-2 font-medium">Attempt</th>
          </tr>
        </thead>
        <tbody>
          {(data?.items ?? []).map((step: StepListItem) => {
            const isExpanded = expandedStep === step.name
            return (
              <React.Fragment key={step.name}>
                <tr
                  className="border-t hover:bg-gray-50 cursor-pointer"
                  onClick={() => setExpandedStep(isExpanded ? null : step.name)}
                >
                  <td className="px-4 py-2 text-gray-400 text-xs">{isExpanded ? '▼' : '▶'}</td>
                  <td className="px-4 py-2 font-mono text-xs">{step.name}</td>
                  <td className="px-4 py-2"><StateBadge state={step.state} /></td>
                  <td className="px-4 py-2 text-gray-500">{step.attempt}</td>
                </tr>
                {isExpanded && (
                  <tr>
                    <td colSpan={4} className="p-0">
                      <StepDetail runId={runId} stepName={step.name} state={step.state} />
                    </td>
                  </tr>
                )}
              </React.Fragment>
            )
          })}
          {!isLoading && !data?.items.length && (
            <tr>
              <td colSpan={4} className="px-4 py-4 text-center text-gray-400 text-xs">No steps yet.</td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  )
}
