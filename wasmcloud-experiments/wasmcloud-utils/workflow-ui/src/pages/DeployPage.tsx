import { useState } from 'react'
import { useQuery, useQueries, useMutation, useQueryClient } from '@tanstack/react-query'
import { listWorkflows, getWorkflow, getWorkflowManifest, createWorkflow } from '../api'
import type { WorkflowDef } from '../api'
import { WorkflowCanvas } from '../components/WorkflowCanvas'

export function DeployPage() {
  const qc = useQueryClient()
  const [selectedWorkflow, setSelectedWorkflow] = useState<string | null>(null)
  const [canvasDef, setCanvasDef] = useState<WorkflowDef | undefined>(undefined)
  const [showCanvas, setShowCanvas] = useState(false)

  const create = useMutation({
    mutationFn: (def: WorkflowDef) => createWorkflow(def),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['workflows'] }),
  })

  const { data: workflowsPage, isLoading } = useQuery({
    queryKey: ['workflows'],
    queryFn: () => listWorkflows(),
  })

  const names = workflowsPage?.items ?? []

  const workflowQueries = useQueries({
    queries: names.map(name => ({
      queryKey: ['workflow', name],
      queryFn: () => getWorkflow(name),
    })),
  })

  const { data: manifest, isLoading: manifestLoading, error: manifestError } = useQuery({
    queryKey: ['manifest', selectedWorkflow],
    queryFn: () => getWorkflowManifest(selectedWorkflow!),
    enabled: selectedWorkflow !== null,
    retry: false,
  })

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  const openNewCanvas = () => {
    setCanvasDef(undefined)
    setShowCanvas(true)
  }

  const openEditCanvas = (def: WorkflowDef) => {
    setCanvasDef(def)
    setShowCanvas(true)
  }

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Deploy</h1>
        <button
          onClick={openNewCanvas}
          className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700"
        >
          + New Workflow (Visual)
        </button>
      </div>

      {showCanvas && (
        <WorkflowCanvas
          initialDef={canvasDef}
          onSave={(def) => create.mutateAsync(def).then(() => {})}
          onClose={() => setShowCanvas(false)}
        />
      )}

      {isLoading ? (
        <p className="text-gray-500 dark:text-gray-400 text-sm">Loading workflows…</p>
      ) : (
        <div className="overflow-x-auto rounded border border-gray-200 dark:border-gray-700">
          <table className="min-w-full text-sm">
            <thead className="bg-gray-100 dark:bg-gray-800 text-left">
              <tr>
                <th className="px-4 py-2 font-medium text-gray-700 dark:text-gray-300">Workflow</th>
                <th className="px-4 py-2 font-medium text-gray-700 dark:text-gray-300">Steps with components</th>
                <th className="px-4 py-2 font-medium text-gray-700 dark:text-gray-300">Action</th>
              </tr>
            </thead>
            <tbody>
              {names.map((name, i) => {
                const wfQuery = workflowQueries[i]
                const def = wfQuery?.data
                const stepsWithComponents = def?.steps.filter(s => s.component) ?? []
                const hasComponents = stepsWithComponents.length > 0

                return (
                  <tr key={name} className="border-t border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800">
                    <td className="px-4 py-2 font-mono font-medium">{name}</td>
                    <td className="px-4 py-2 text-gray-600 dark:text-gray-400">
                      {wfQuery?.isLoading ? (
                        <span className="text-gray-400 dark:text-gray-500 italic">loading…</span>
                      ) : hasComponents ? (
                        stepsWithComponents.map(s => (
                          <span key={s.name} className="inline-block mr-2">
                            <span className="font-mono">{s.name}</span>
                            <span className="text-gray-400 dark:text-gray-500 ml-1">({s.component})</span>
                          </span>
                        ))
                      ) : (
                        <span className="text-gray-400 dark:text-gray-500 italic">no components</span>
                      )}
                    </td>
                    <td className="px-4 py-2 space-x-2">
                      {def && (
                        <button
                          onClick={() => openEditCanvas(def)}
                          className="px-3 py-1 bg-purple-100 dark:bg-purple-900 text-purple-800 dark:text-purple-300 rounded text-xs hover:bg-purple-200 dark:hover:bg-purple-800"
                        >
                          Edit (Visual)
                        </button>
                      )}
                      {hasComponents && (
                        <>
                          <button
                            onClick={() => setSelectedWorkflow(selectedWorkflow === name ? null : name)}
                            className="px-3 py-1 bg-blue-600 text-white rounded text-xs hover:bg-blue-700"
                          >
                            {selectedWorkflow === name ? 'Close' : 'View Manifest'}
                          </button>
                          {selectedWorkflow === name && manifest && (
                            <button
                              onClick={() => copyToClipboard(manifest)}
                              className="px-3 py-1 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded text-xs hover:bg-gray-300 dark:hover:bg-gray-600"
                            >
                              Copy
                            </button>
                          )}
                        </>
                      )}
                    </td>
                  </tr>
                )
              })}
              {names.length === 0 && (
                <tr>
                  <td colSpan={3} className="px-4 py-6 text-center text-gray-400 dark:text-gray-500 italic">
                    No workflows found.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {selectedWorkflow && (
        <div className="mt-4 border border-gray-200 dark:border-gray-700 rounded p-4 bg-white dark:bg-gray-900 space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="font-semibold text-gray-800 dark:text-gray-200">
              WADM Manifest — <span className="font-mono">{selectedWorkflow}</span>
            </h2>
            <div className="flex gap-2">
              {manifest && (
                <button
                  onClick={() => copyToClipboard(manifest)}
                  className="px-3 py-1 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded text-xs hover:bg-gray-300 dark:hover:bg-gray-600"
                >
                  Copy to clipboard
                </button>
              )}
              <button
                onClick={() => setSelectedWorkflow(null)}
                className="px-3 py-1 bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300 rounded text-xs hover:bg-red-200 dark:hover:bg-red-800"
              >
                Close
              </button>
            </div>
          </div>

          {manifestLoading && <p className="text-gray-500 dark:text-gray-400 text-sm">Loading manifest…</p>}
          {manifestError && (
            <p className="text-red-600 dark:text-red-400 text-sm">
              {(manifestError as Error).message}
            </p>
          )}
          {manifest && (
            <pre className="bg-gray-900 text-green-300 text-xs p-4 rounded overflow-x-auto whitespace-pre font-mono">
              {manifest}
            </pre>
          )}
        </div>
      )}
    </div>
  )
}
