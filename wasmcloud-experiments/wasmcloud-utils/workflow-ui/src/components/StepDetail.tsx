import { useQuery, useQueryClient } from '@tanstack/react-query'
import { getStepOutput } from '../api'
import { StepActions } from './StepActions'

interface Props {
  runId: string
  stepName: string
  state: string
}

export function StepDetail({ runId, stepName, state }: Props) {
  const qc = useQueryClient()

  const { data, isLoading } = useQuery({
    queryKey: ['step-output', runId, stepName],
    queryFn: () => getStepOutput(runId, stepName),
  })

  const onAction = () => {
    qc.invalidateQueries({ queryKey: ['steps', runId] })
    qc.invalidateQueries({ queryKey: ['step-output', runId, stepName] })
  }

  return (
    <div className="bg-gray-50 border-t border-gray-200 px-6 py-4 space-y-3">
      <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide">Output</div>
      {isLoading ? (
        <p className="text-xs text-gray-400">Loading…</p>
      ) : (
        <pre className="bg-white border rounded p-3 text-xs font-mono overflow-auto max-h-48 whitespace-pre-wrap">
          {data?.output != null ? JSON.stringify(data.output, null, 2) : '—'}
        </pre>
      )}
      {state === 'failed' && (
        <div className="text-xs text-red-600 font-medium">State: failed</div>
      )}
      <StepActions runId={runId} stepName={stepName} state={state} onAction={onAction} />
    </div>
  )
}
