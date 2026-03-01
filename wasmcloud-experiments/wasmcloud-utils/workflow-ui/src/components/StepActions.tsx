import { useState } from 'react'
import { markStepDone, markStepFailed, retryStep } from '../api'

interface Props {
  runId: string
  stepName: string
  state: string
  onAction: () => void
}

export function StepActions({ runId, stepName, state, onAction }: Props) {
  const [showOutput, setShowOutput] = useState(false)
  const [outputText, setOutputText] = useState('')
  const [loading, setLoading] = useState(false)

  const act = async (fn: () => Promise<void>) => {
    setLoading(true)
    try { await fn() } catch { /* ignore */ } finally {
      setLoading(false)
      onAction()
    }
  }

  const done = () => {
    let output: unknown
    if (outputText.trim()) {
      try { output = JSON.parse(outputText) } catch { output = outputText }
    }
    return act(() => markStepDone(runId, stepName, output))
  }

  if (state === 'succeeded' || state === 'skipped') {
    return <span className="text-gray-400 text-xs">—</span>
  }

  return (
    <div className="flex gap-1 items-center">
      {showOutput && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl w-full max-w-md p-6 space-y-3">
            <h3 className="font-semibold">Output JSON (optional)</h3>
            <textarea
              className="w-full h-32 font-mono text-sm border rounded p-2"
              placeholder='{"result": "ok"}'
              value={outputText}
              onChange={(e) => setOutputText(e.target.value)}
            />
            <div className="flex justify-end gap-2">
              <button onClick={() => setShowOutput(false)} className="px-3 py-1 border rounded text-sm">Cancel</button>
              <button onClick={() => { setShowOutput(false); done() }} className="px-3 py-1 bg-green-600 text-white rounded text-sm">Done</button>
            </div>
          </div>
        </div>
      )}
      <button
        disabled={loading}
        onClick={() => setShowOutput(true)}
        className="px-2 py-0.5 bg-green-100 text-green-800 rounded text-xs hover:bg-green-200 disabled:opacity-50"
      >Done</button>
      <button
        disabled={loading}
        onClick={() => act(() => markStepFailed(runId, stepName))}
        className="px-2 py-0.5 bg-red-100 text-red-800 rounded text-xs hover:bg-red-200 disabled:opacity-50"
      >Fail</button>
      {state === 'failed' && (
        <button
          disabled={loading}
          onClick={() => act(() => retryStep(runId, stepName))}
          className="px-2 py-0.5 bg-yellow-100 text-yellow-800 rounded text-xs hover:bg-yellow-200 disabled:opacity-50"
        >Retry</button>
      )}
    </div>
  )
}
