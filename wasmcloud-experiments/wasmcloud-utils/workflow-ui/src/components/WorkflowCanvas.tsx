import { createContext, useCallback, useContext, useEffect, useRef, useState } from 'react'
import { YamlEditor } from './YamlEditor'
import {
  ReactFlow,
  addEdge,
  applyNodeChanges,
  applyEdgeChanges,
  Background,
  Controls,
  MiniMap,
  Handle,
  Position,
  type Node,
  type Edge,
  type Connection,
  type NodeChange,
  type EdgeChange,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import type { WorkflowDef, StepDef } from '../api'

// ── Node data (no callbacks — those come via context) ─────────────────────────

interface StepNodeData extends Record<string, unknown> {
  label: string
  component: string
  max_attempts: number
  timeout_ms: number | null
  optional: boolean
}

// ── Stable callback context ───────────────────────────────────────────────────

interface NodeCbs {
  onDelete: (id: string) => void
  onChange: (id: string, field: string, value: unknown) => void
}
const NodeCbsCtx = createContext<NodeCbs>({
  onDelete: () => {},
  onChange: () => {},
})

// ── StepNode card ─────────────────────────────────────────────────────────────

function StepNode({ id, data }: { id: string; data: StepNodeData }) {
  const { onDelete, onChange } = useContext(NodeCbsCtx)
  return (
    <div className="bg-white border border-gray-300 rounded shadow-md w-56 text-xs font-sans select-none">
      <Handle type="target" position={Position.Left} className="!bg-blue-400 !w-3 !h-3" />
      <div className="flex items-center justify-between bg-gray-100 px-2 py-1 rounded-t border-b border-gray-200">
        <input
          className="flex-1 font-mono text-xs bg-transparent outline-none font-semibold min-w-0"
          value={data.label}
          onChange={e => onChange(id, 'label', e.target.value)}
          placeholder="step-name"
        />
        <button
          onMouseDown={e => e.stopPropagation()}
          onClick={() => onDelete(id)}
          className="ml-1 text-gray-400 hover:text-red-500 font-bold leading-none shrink-0"
        >
          ×
        </button>
      </div>
      <div className="px-2 py-2 space-y-1">
        <div>
          <div className="text-gray-400 mb-0.5">OCI image</div>
          <input
            className="w-full border border-gray-200 rounded px-1 py-0.5 text-xs font-mono"
            value={data.component}
            onChange={e => onChange(id, 'component', e.target.value)}
            placeholder="ghcr.io/org/step:v1"
          />
        </div>
        <div className="flex gap-2 items-center pt-0.5">
          <label className="text-gray-400 flex items-center gap-1 cursor-pointer">
            <input
              type="checkbox"
              checked={data.optional}
              onChange={e => onChange(id, 'optional', e.target.checked)}
            />
            optional
          </label>
          <label className="text-gray-400 ml-auto flex items-center gap-1">
            attempts
            <input
              type="number"
              min={1}
              className="w-10 border border-gray-200 rounded px-1 py-0.5 text-xs text-center"
              value={data.max_attempts}
              onChange={e => onChange(id, 'max_attempts', Number(e.target.value))}
            />
          </label>
        </div>
      </div>
      <Handle type="source" position={Position.Right} className="!bg-green-400 !w-3 !h-3" />
    </div>
  )
}

// nodeTypes must be stable (defined outside the component)
const nodeTypes = { stepNode: StepNode }

// ── Auto-layout ───────────────────────────────────────────────────────────────

function autoLayout(steps: StepDef[]): Record<string, { x: number; y: number }> {
  const COL_W = 280
  const ROW_H = 160
  const depth: Record<string, number> = {}
  const roots = steps.filter(s => s.depends_on.length === 0).map(s => s.name)
  const queue = [...roots]
  for (const r of roots) depth[r] = 0
  while (queue.length) {
    const cur = queue.shift()!
    for (const s of steps) {
      if (s.depends_on.includes(cur)) {
        depth[s.name] = Math.max(depth[s.name] ?? 0, (depth[cur] ?? 0) + 1)
        queue.push(s.name)
      }
    }
  }
  for (const s of steps) if (depth[s.name] === undefined) depth[s.name] = 0
  const colCounts: Record<number, number> = {}
  const positions: Record<string, { x: number; y: number }> = {}
  for (const s of steps) {
    const col = depth[s.name]
    const row = colCounts[col] ?? 0
    colCounts[col] = row + 1
    positions[s.name] = { x: col * COL_W + 40, y: row * ROW_H + 40 }
  }
  return positions
}

// ── WorkflowDef ↔ ReactFlow ───────────────────────────────────────────────────

function fromDefToFlow(def: WorkflowDef) {
  const positions = autoLayout(def.steps)
  const nodes: Node[] = def.steps.map(s => ({
    id: s.name,
    type: 'stepNode',
    position: positions[s.name] ?? { x: 0, y: 0 },
    data: {
      label: s.name,
      component: s.component ?? '',
      max_attempts: s.max_attempts,
      timeout_ms: s.timeout_ms,
      optional: s.optional,
    } satisfies StepNodeData,
  }))
  const edges: Edge[] = def.steps.flatMap(s =>
    s.depends_on.map(dep => ({
      id: `${dep}->${s.name}`,
      source: dep,
      target: s.name,
    }))
  )
  return { nodes, edges }
}

function toWorkflowDef(wfName: string, nodes: Node[], edges: Edge[]): WorkflowDef {
  const depsMap: Record<string, string[]> = {}
  for (const e of edges) {
    depsMap[e.target] = [...(depsMap[e.target] ?? []), e.source]
  }
  return {
    name: wfName,
    description: null,
    timeout_ms: null,
    triggers: [],
    steps: nodes.map(n => {
      const d = n.data as StepNodeData
      return {
        name: d.label || n.id,
        depends_on: depsMap[n.id] ?? [],
        component: d.component || null,
        max_attempts: d.max_attempts ?? 1,
        base_delay_ms: 0,
        timeout_ms: d.timeout_ms ?? null,
        sub_workflow: null,
        optional: d.optional ?? false,
        condition: null,
      }
    }),
  }
}

// ── WADM YAML generation ──────────────────────────────────────────────────────

function toWadmYaml(def: WorkflowDef): string {
  const lines: string[] = [
    `apiVersion: core.oam.dev/v1beta1`,
    `kind: Application`,
    `metadata:`,
    `  name: ${def.name || 'my-workflow'}`,
    `  annotations:`,
    `    description: Workflow ${def.name || 'my-workflow'}`,
    `spec:`,
    `  components:`,
  ]
  for (const step of def.steps) {
    if (!step.component) continue
    const compName = step.name.replace(/[^a-z0-9-]/gi, '-')
    lines.push(
      `    - name: ${compName}`,
      `      type: component`,
      `      properties:`,
      `        image: ${step.component}`,
      `      traits:`,
      `        - type: spreadscaler`,
      `          properties:`,
      `            replicas: 1`,
    )
    if (step.depends_on.length > 0) {
      lines.push(
        `        - type: link`,
        `          properties:`,
        `            namespace: default`,
        `            package: workflow`,
        `            interfaces: [step]`,
        `            target:`,
        `              name: workflow-engine`,
      )
    }
  }
  if (!def.steps.some(s => s.component)) {
    lines.push(`    # No OCI components set — add images to steps to generate component entries`)
  }
  lines.push(``, `  # Workflow definition (used by the wasmCloud workflow engine)`, `  # steps:`)
  for (const step of def.steps) {
    lines.push(`  #   - name: ${step.name}`)
    if (step.depends_on.length > 0) lines.push(`  #     depends_on: [${step.depends_on.join(', ')}]`)
    if (step.component) lines.push(`  #     component: ${step.component}`)
    if (step.optional) lines.push(`  #     optional: true`)
    if (step.max_attempts > 1) lines.push(`  #     max_attempts: ${step.max_attempts}`)
  }
  return lines.join('\n')
}

// ── Parse steps from YAML comment block ──────────────────────────────────────

function parseYamlCommentSteps(yaml: string, wfName: string): WorkflowDef {
  const steps: StepDef[] = []
  let cur: Partial<StepDef> | null = null
  for (const raw of yaml.split('\n')) {
    const line = raw.replace(/^#\s?/, '').trimEnd()
    const nameMatch = line.match(/^\s{2,4}- name:\s+(.+)$/)
    if (nameMatch) {
      if (cur?.name) steps.push(makeStep(cur))
      cur = { name: nameMatch[1].trim(), depends_on: [] }
      continue
    }
    if (!cur) continue
    const depsMatch = line.match(/^\s+depends_on:\s*\[(.+)\]/)
    if (depsMatch) { cur.depends_on = depsMatch[1].split(',').map(s => s.trim()).filter(Boolean); continue }
    const compMatch = line.match(/^\s+component:\s+(.+)$/)
    if (compMatch) { cur.component = compMatch[1].trim(); continue }
    const optMatch = line.match(/^\s+optional:\s+(true|false)/)
    if (optMatch) { cur.optional = optMatch[1] === 'true'; continue }
    const attMatch = line.match(/^\s+max_attempts:\s+(\d+)/)
    if (attMatch) { cur.max_attempts = parseInt(attMatch[1]); continue }
  }
  if (cur?.name) steps.push(makeStep(cur))
  return { name: wfName, description: null, timeout_ms: null, triggers: [], steps }
}

function makeStep(p: Partial<StepDef>): StepDef {
  return {
    name: p.name!, depends_on: p.depends_on ?? [], component: p.component ?? null,
    max_attempts: p.max_attempts ?? 1, base_delay_ms: 0, timeout_ms: null,
    sub_workflow: null, optional: p.optional ?? false, condition: null,
  }
}

// ── Props ─────────────────────────────────────────────────────────────────────

interface Props {
  initialDef?: WorkflowDef
  onSave: (def: WorkflowDef) => Promise<void>
  onClose: () => void
}

// ── Main component ────────────────────────────────────────────────────────────

export function WorkflowCanvas({ initialDef, onSave, onClose }: Props) {
  const [wfName, setWfName] = useState(initialDef?.name ?? '')
  const [nodes, setNodes] = useState<Node[]>([])
  const [edges, setEdges] = useState<Edge[]>([])
  const [wadmYaml, setWadmYaml] = useState('')
  const [yamlError, setYamlError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [saveError, setSaveError] = useState<string | null>(null)

  // Suppresses the canvas→yaml effect when yaml is the source of truth
  const suppressCanvasSync = useRef(false)

  // ── Stable node callbacks via context (never embedded in node.data) ─────────

  const onDelete = useCallback(
    (id: string) => setNodes(ns => ns.filter(n => n.id !== id)),
    []
  )
  const onChange = useCallback(
    (id: string, field: string, value: unknown) =>
      setNodes(ns => ns.map(n => n.id === id ? { ...n, data: { ...n.data, [field]: value } } : n)),
    []
  )
  const nodeCbs: NodeCbs = { onDelete, onChange }

  // ── Init ───────────────────────────────────────────────────────────────────

  useEffect(() => {
    const def = initialDef ?? {
      name: '', description: null, timeout_ms: null, triggers: [],
      steps: [{
        name: 'step-1', depends_on: [], component: null,
        max_attempts: 1, base_delay_ms: 0, timeout_ms: null,
        sub_workflow: null, optional: false, condition: null,
      }],
    }
    const { nodes: ns, edges: es } = fromDefToFlow(def)
    setNodes(ns)
    setEdges(es)
    setWfName(def.name)
    setWadmYaml(toWadmYaml(def))
  }, [initialDef])

  // ── Canvas → YAML sync ─────────────────────────────────────────────────────

  useEffect(() => {
    if (suppressCanvasSync.current) return
    const def = toWorkflowDef(wfName, nodes, edges)
    setWadmYaml(toWadmYaml(def))
    setYamlError(null)
  }, [nodes, edges, wfName])

  // ── ReactFlow handlers ─────────────────────────────────────────────────────

  const onNodesChange = useCallback(
    (changes: NodeChange[]) => setNodes(ns => applyNodeChanges(changes, ns)),
    []
  )
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges(es => applyEdgeChanges(changes, es)),
    []
  )
  const onConnect = useCallback(
    (connection: Connection) =>
      setEdges(es => addEdge({ ...connection, id: `${connection.source}->${connection.target}` }, es)),
    []
  )

  // ── YAML → canvas sync ─────────────────────────────────────────────────────

  const handleYamlChange = (text: string) => {
    setWadmYaml(text)
    setYamlError(null)
    try {
      const def = parseYamlCommentSteps(text, wfName)
      if (def.steps.length > 0) {
        suppressCanvasSync.current = true
        const { nodes: ns, edges: es } = fromDefToFlow(def)
        setNodes(ns)
        setEdges(es)
        setTimeout(() => { suppressCanvasSync.current = false }, 0)
      }
    } catch {
      // leave graph as-is while user is mid-edit
    }
  }

  // ── Add step ───────────────────────────────────────────────────────────────

  const addStep = () => {
    const id = `step-${Date.now()}`
    setNodes(ns => [
      ...ns,
      {
        id,
        type: 'stepNode',
        position: { x: ns.length * 280 + 40, y: 40 },
        data: { label: id, component: '', max_attempts: 1, timeout_ms: null, optional: false } satisfies StepNodeData,
      },
    ])
  }

  // ── Save ───────────────────────────────────────────────────────────────────

  const handleSave = async () => {
    setSaveError(null)
    setSaving(true)
    try {
      const def = toWorkflowDef(wfName, nodes, edges)
      if (!def.name) { setSaveError('Workflow name is required'); setSaving(false); return }
      await onSave(def)
      onClose()
    } catch (e) {
      setSaveError((e as Error).message)
    } finally {
      setSaving(false)
    }
  }

  // ── Render ─────────────────────────────────────────────────────────────────

  return (
    <NodeCbsCtx.Provider value={nodeCbs}>
      <div className="fixed inset-0 z-50 flex flex-col bg-white">
        {/* toolbar */}
        <div className="flex items-center gap-3 px-4 py-2 border-b bg-gray-50 shrink-0">
          <span className="font-semibold text-gray-800 text-sm">Workflow Builder</span>
          <input
            className="border border-gray-300 rounded px-2 py-1 text-sm font-mono w-48"
            value={wfName}
            onChange={e => setWfName(e.target.value)}
            placeholder="workflow-name"
          />
          <button
            onClick={addStep}
            className="px-3 py-1 bg-green-600 text-white rounded text-sm hover:bg-green-700"
          >
            + Add Step
          </button>
          <div className="ml-auto flex items-center gap-2">
            {saveError && <span className="text-red-600 text-xs">{saveError}</span>}
            <button
              onClick={handleSave}
              disabled={saving}
              className="px-4 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50"
            >
              {saving ? 'Saving…' : 'Save Workflow'}
            </button>
            <button
              onClick={onClose}
              className="px-3 py-1 bg-gray-200 text-gray-700 rounded text-sm hover:bg-gray-300"
            >
              Cancel
            </button>
          </div>
        </div>

        {/* split body */}
        <div className="flex flex-1 overflow-hidden">
          {/* LEFT — ReactFlow canvas */}
          <div className="flex-1 min-w-0 relative border-r border-gray-200">
            <div className="absolute top-2 left-2 z-10 bg-white/80 text-xs text-gray-400 px-2 py-1 rounded pointer-events-none">
              Drag green → blue handle to connect steps · Delete key removes selected
            </div>
            <ReactFlow
              nodes={nodes}
              edges={edges}
              onNodesChange={onNodesChange}
              onEdgesChange={onEdgesChange}
              onConnect={onConnect}
              nodeTypes={nodeTypes}
              fitView
              deleteKeyCode="Delete"
            >
              <Background />
              <Controls />
              <MiniMap nodeStrokeWidth={3} />
            </ReactFlow>
          </div>

          {/* RIGHT — WADM YAML with syntax highlighting + vim */}
          <div className="w-[480px] shrink-0 flex flex-col bg-gray-950">
            <div className="flex items-center justify-between px-3 py-2 border-b border-gray-800 shrink-0">
              <span className="text-xs font-mono text-gray-400">WADM manifest</span>
              <button
                onClick={() => navigator.clipboard.writeText(wadmYaml)}
                className="text-xs px-2 py-0.5 bg-gray-700 text-gray-200 rounded hover:bg-gray-600"
              >
                Copy
              </button>
            </div>
            {yamlError && (
              <div className="px-3 py-1 text-xs text-red-400 bg-red-950/40 border-b border-red-900 shrink-0">
                {yamlError}
              </div>
            )}
            <div className="flex-1 overflow-hidden">
              <YamlEditor
                value={wadmYaml}
                onChange={handleYamlChange}
                onSave={handleSave}
              />
            </div>
          </div>
        </div>
      </div>
    </NodeCbsCtx.Provider>
  )
}
