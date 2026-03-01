import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { listWorkflows, createWorkflow, deleteWorkflow, startRun } from '../api'
import type { WorkflowDef } from '../api'
import { WorkflowEditor } from '../components/WorkflowEditor'
import { WorkflowCanvas } from '../components/WorkflowCanvas'

export function WorkflowsPage() {
  const qc = useQueryClient()
  const [showEditor, setShowEditor] = useState(false)
  const [showCanvas, setShowCanvas] = useState(false)

  const { data, isLoading } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => listWorkflows(),
    refetchInterval: 10_000,
  })

  const create = useMutation({
    mutationFn: (def: WorkflowDef) => createWorkflow(def),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['workflows'] }),
  })

  const del = useMutation({
    mutationFn: (name: string) => deleteWorkflow(name),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['workflows'] }),
  })

  const start = useMutation({
    mutationFn: (name: string) => startRun(name),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['runs'] }),
  })

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Workflows</h1>
        <div className="flex gap-2">
          <button
            onClick={() => setShowCanvas(true)}
            className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700"
          >
            + Visual Editor
          </button>
          <button
            onClick={() => setShowEditor(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
          >
            + New Workflow
          </button>
        </div>
      </div>

      {showEditor && (
        <WorkflowEditor
          onSave={(def) => create.mutateAsync(def).then(() => {})}
          onClose={() => setShowEditor(false)}
        />
      )}

      {showCanvas && (
        <WorkflowCanvas
          onSave={(def) => create.mutateAsync(def).then(() => {})}
          onClose={() => setShowCanvas(false)}
        />
      )}

      {isLoading && <p className="text-gray-500">Loading…</p>}

      <table className="w-full text-sm border rounded overflow-hidden">
        <thead className="bg-gray-50 text-left">
          <tr>
            <th className="px-4 py-2 font-medium">Name</th>
            <th className="px-4 py-2 font-medium text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          {(data?.items ?? []).map((name) => (
            <tr key={name} className="border-t hover:bg-gray-50">
              <td className="px-4 py-2 font-mono">{name}</td>
              <td className="px-4 py-2 flex justify-end gap-2">
                <button
                  onClick={() => start.mutate(name)}
                  className="px-3 py-1 bg-green-100 text-green-800 rounded text-xs hover:bg-green-200"
                >
                  ▶ Start Run
                </button>
                <button
                  onClick={() => {
                    if (confirm(`Delete workflow "${name}"?`)) del.mutate(name)
                  }}
                  className="px-3 py-1 bg-red-100 text-red-800 rounded text-xs hover:bg-red-200"
                >
                  Delete
                </button>
              </td>
            </tr>
          ))}
          {!isLoading && data?.items.length === 0 && (
            <tr>
              <td colSpan={2} className="px-4 py-6 text-center text-gray-400">
                No workflows yet. Create one above.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  )
}
