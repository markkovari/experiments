import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  listSecrets,
  getSecretMeta,
  setSecret,
  deleteSecret,
  rotateSecret,
  type SecretMetadata,
} from '../secrets-api'

// ── helpers ────────────────────────────────────────────────────────────────

function fmtTs(ms: number) {
  if (ms === 0) return '—'
  return new Date(ms).toLocaleString()
}

// ── Add / Rotate modal ─────────────────────────────────────────────────────

interface SecretFormProps {
  title: string
  nameLocked?: string
  onSubmit: (name: string, value: string) => Promise<void>
  onClose: () => void
  submitLabel: string
  submitClass: string
}

function SecretForm({ title, nameLocked, onSubmit, onClose, submitLabel, submitClass }: SecretFormProps) {
  const [name, setName] = useState(nameLocked ?? '')
  const [value, setValue] = useState('')
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  async function handle(e: React.FormEvent) {
    e.preventDefault()
    if (!name.trim() || !value) return
    setBusy(true)
    setErr(null)
    try {
      await onSubmit(name.trim(), value)
      onClose()
    } catch (ex) {
      setErr(ex instanceof Error ? ex.message : String(ex))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="fixed inset-0 bg-black/40 dark:bg-black/60 flex items-center justify-center z-50">
      <form
        onSubmit={handle}
        className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl w-full max-w-md p-6 space-y-4"
      >
        <h2 className="text-lg font-semibold">{title}</h2>

        <div className="space-y-1">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Name</label>
          <input
            className="w-full border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm font-mono bg-white dark:bg-gray-800 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-400 disabled:bg-gray-100 dark:disabled:bg-gray-700"
            value={name}
            onChange={(e) => setName(e.target.value)}
            disabled={!!nameLocked}
            placeholder="my-db-password"
            required
          />
        </div>

        <div className="space-y-1">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Value</label>
          <textarea
            className="w-full border border-gray-300 dark:border-gray-600 rounded px-3 py-2 text-sm font-mono bg-white dark:bg-gray-800 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-400 resize-none"
            rows={4}
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder="Paste the secret value here (will be AES-256 encrypted)"
            required
          />
          <p className="text-xs text-gray-400 dark:text-gray-500">
            The value is Base64-encoded in the browser and encrypted at rest with AES-256-GCM.
            It is never stored or logged in plaintext.
          </p>
        </div>

        {err && (
          <p className="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded px-3 py-2">
            {err}
          </p>
        )}

        <div className="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm rounded border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-800"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={busy}
            className={`px-4 py-2 text-sm rounded text-white disabled:opacity-60 ${submitClass}`}
          >
            {busy ? 'Saving…' : submitLabel}
          </button>
        </div>
      </form>
    </div>
  )
}

// ── Detail panel ───────────────────────────────────────────────────────────

interface DetailPanelProps {
  name: string
  onRotate: () => void
  onClose: () => void
}

function DetailPanel({ name, onRotate, onClose }: DetailPanelProps) {
  const { data, isLoading, error } = useQuery<SecretMetadata>({
    queryKey: ['secret', name],
    queryFn: () => getSecretMeta(name),
  })

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded bg-gray-50 dark:bg-gray-800 p-4 space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="font-semibold font-mono">{name}</h3>
        <button onClick={onClose} className="text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 text-lg leading-none">×</button>
      </div>

      {isLoading && <p className="text-sm text-gray-500 dark:text-gray-400">Loading metadata…</p>}
      {error && (
        <p className="text-sm text-red-600 dark:text-red-400">
          {error instanceof Error ? error.message : 'Error loading metadata'}
        </p>
      )}

      {data && (
        <dl className="text-sm grid grid-cols-2 gap-x-4 gap-y-1">
          <dt className="text-gray-500 dark:text-gray-400">Version</dt>
          <dd className="font-mono">{data.version}</dd>
          <dt className="text-gray-500 dark:text-gray-400">Created</dt>
          <dd>{fmtTs(data.created_at_ms)}</dd>
          <dt className="text-gray-500 dark:text-gray-400">Updated</dt>
          <dd>{fmtTs(data.updated_at_ms)}</dd>
        </dl>
      )}

      <div className="flex gap-2 pt-1">
        <button
          onClick={onRotate}
          className="px-3 py-1 text-xs bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-300 rounded hover:bg-yellow-200 dark:hover:bg-yellow-800"
        >
          ↻ Rotate
        </button>
      </div>
    </div>
  )
}

// ── Main page ──────────────────────────────────────────────────────────────

export function SecretsPage() {
  const qc = useQueryClient()
  const [selected, setSelected] = useState<string | null>(null)
  const [showAdd, setShowAdd] = useState(false)
  const [rotating, setRotating] = useState<string | null>(null)

  const { data: names, isLoading, error } = useQuery<string[]>({
    queryKey: ['secrets'],
    queryFn: listSecrets,
    refetchInterval: 15_000,
  })

  const add = useMutation({
    mutationFn: ({ name, value }: { name: string; value: string }) => setSecret(name, value),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['secrets'] }),
  })

  const del = useMutation({
    mutationFn: (name: string) => deleteSecret(name),
    onSuccess: (_: { ok: boolean }, name: string) => {
      if (selected === name) setSelected(null)
      qc.invalidateQueries({ queryKey: ['secrets'] })
    },
  })

  const rotate = useMutation({
    mutationFn: ({ name, value }: { name: string; value: string }) => rotateSecret(name, value),
    onSuccess: (_: SecretMetadata, { name }: { name: string; value: string }) => qc.invalidateQueries({ queryKey: ['secret', name] }),
  })

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Secrets</h1>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
            Stored encrypted (AES-256-GCM) in NATS KV. Values are never displayed.
          </p>
        </div>
        <button
          onClick={() => setShowAdd(true)}
          className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
        >
          + Add Secret
        </button>
      </div>

      {showAdd && (
        <SecretForm
          title="Add Secret"
          onSubmit={(name, value) => add.mutateAsync({ name, value }).then(() => {})}
          onClose={() => setShowAdd(false)}
          submitLabel="Save Secret"
          submitClass="bg-blue-600 hover:bg-blue-700"
        />
      )}

      {rotating && (
        <SecretForm
          title={`Rotate "${rotating}"`}
          nameLocked={rotating}
          onSubmit={(_, value) => rotate.mutateAsync({ name: rotating, value }).then(() => {})}
          onClose={() => setRotating(null)}
          submitLabel="Rotate Secret"
          submitClass="bg-yellow-600 hover:bg-yellow-700"
        />
      )}

      {error && (
        <div className="bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700 rounded px-4 py-3 text-sm text-red-700 dark:text-red-400">
          Could not reach secrets-http component: {error instanceof Error ? error.message : String(error)}
          <br />
          <span className="text-xs text-red-500 dark:text-red-500">
            Make sure wasmCloud is running and secrets-http is deployed (wadm/secrets-kv.yaml).
          </span>
        </div>
      )}

      <div className="flex gap-4 items-start">
        <div className="flex-1 min-w-0">
          {isLoading && <p className="text-gray-500 dark:text-gray-400 text-sm">Loading…</p>}
          <table className="w-full text-sm border border-gray-200 dark:border-gray-700 rounded overflow-hidden">
            <thead className="bg-gray-50 dark:bg-gray-800 text-left">
              <tr>
                <th className="px-4 py-2 font-medium">Name</th>
                <th className="px-4 py-2 font-medium text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {(names ?? []).map((name) => (
                <tr
                  key={name}
                  className={`border-t border-gray-200 dark:border-gray-700 cursor-pointer ${
                    selected === name
                      ? 'bg-blue-50 dark:bg-blue-900/30'
                      : 'hover:bg-gray-50 dark:hover:bg-gray-800'
                  }`}
                  onClick={() => setSelected(selected === name ? null : name)}
                >
                  <td className="px-4 py-2 font-mono">{name}</td>
                  <td className="px-4 py-2 text-right" onClick={(e) => e.stopPropagation()}>
                    <div className="flex justify-end gap-2">
                      <button
                        onClick={() => setRotating(name)}
                        className="px-3 py-1 bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-300 rounded text-xs hover:bg-yellow-200 dark:hover:bg-yellow-800"
                      >
                        ↻ Rotate
                      </button>
                      <button
                        onClick={() => {
                          if (confirm(`Delete secret "${name}"? This cannot be undone.`)) {
                            del.mutate(name)
                          }
                        }}
                        className="px-3 py-1 bg-red-100 dark:bg-red-900 text-red-800 dark:text-red-300 rounded text-xs hover:bg-red-200 dark:hover:bg-red-800"
                      >
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
              {!isLoading && (names ?? []).length === 0 && (
                <tr>
                  <td colSpan={2} className="px-4 py-8 text-center text-gray-400 dark:text-gray-500">
                    No secrets yet. Click "Add Secret" to create one.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {selected && (
          <div className="w-72 shrink-0">
            <DetailPanel
              name={selected}
              onRotate={() => setRotating(selected)}
              onClose={() => setSelected(null)}
            />
          </div>
        )}
      </div>
    </div>
  )
}
