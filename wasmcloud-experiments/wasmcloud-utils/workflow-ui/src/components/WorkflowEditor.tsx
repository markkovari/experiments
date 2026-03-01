import { useState } from 'react'
import type { WorkflowDef } from '../api'

interface Props {
  onSave: (def: WorkflowDef) => Promise<void>
  onClose: () => void
}

const TEMPLATE = JSON.stringify(
  {
    name: 'my-workflow',
    steps: [{ name: 'step-1', depends_on: [], component: 'ghcr.io/org/my-step:v1' }],
  },
  null,
  2,
)

export function WorkflowEditor({ onSave, onClose }: Props) {
  const [text, setText] = useState(TEMPLATE)
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

  const handleSave = async () => {
    setError(null)
    let def: WorkflowDef
    try {
      def = JSON.parse(text) as WorkflowDef
    } catch (e) {
      setError(`JSON parse error: ${(e as Error).message}`)
      return
    }
    setSaving(true)
    try {
      await onSave(def)
      onClose()
    } catch (e) {
      setError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl p-6 space-y-4">
        <h2 className="text-lg font-semibold">New Workflow</h2>
        <textarea
          className="w-full h-64 font-mono text-sm border rounded p-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
        {error && <p className="text-red-600 text-sm">{error}</p>}
        <div className="flex justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded border text-sm hover:bg-gray-50"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 rounded bg-blue-600 text-white text-sm hover:bg-blue-700 disabled:opacity-50"
          >
            {saving ? 'Saving…' : 'Save'}
          </button>
        </div>
      </div>
    </div>
  )
}
