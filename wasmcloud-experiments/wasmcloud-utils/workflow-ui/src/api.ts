// Domain types are generated from Rust structs via ts-rs.
// Regenerate: cargo test export_bindings -p workflow-api
export type { WorkflowDef } from './generated/WorkflowDef'
export type { StepDef } from './generated/StepDef'
export type { TriggerDef } from './generated/TriggerDef'
export type { RunRecord } from './generated/RunRecord'
export type { Condition } from './generated/Condition'

import type { WorkflowDef } from './generated/WorkflowDef'
import type { RunRecord } from './generated/RunRecord'

export const BASE_URL = '/api'

// UI-only: the list steps response adds `name` to each StepRecord row
export interface StepListItem {
  name: string
  state: string
  attempt: number
}

export interface PagedResult<T> {
  items: T[]
  total: number
  page: number
  limit: number
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE_URL}${path}`, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`${method} ${path} → ${res.status}: ${text}`)
  }
  if (res.status === 204) return undefined as unknown as T
  return res.json() as Promise<T>
}

// Workflows
export const listWorkflows = (page = 1, limit = 50) =>
  request<PagedResult<string>>('GET', `/workflows?page=${page}&limit=${limit}`)

export const getWorkflow = (name: string) =>
  request<WorkflowDef>('GET', `/workflows/${name}`)

export const createWorkflow = (def: WorkflowDef) =>
  request<{ name: string; created: boolean }>('POST', '/workflows', def)

export const deleteWorkflow = (name: string) =>
  request<void>('DELETE', `/workflows/${name}`)

export const startRun = (wfName: string, idempotencyKey?: string) =>
  request<{ run_id: string }>('POST', `/workflows/${wfName}/run`,
    idempotencyKey ? { idem_key: idempotencyKey } : {})

// Runs
export const listRuns = (wfName: string, state?: string, page = 1, limit = 50) => {
  const qs = [`page=${page}`, `limit=${limit}`, state ? `state=${state}` : '']
    .filter(Boolean).join('&')
  return request<PagedResult<RunRecord>>('GET', `/workflows/${wfName}/runs?${qs}`)
}

export const getRun = (runId: string) =>
  request<RunRecord>('GET', `/runs/${runId}`)

export const cancelRun = (runId: string) =>
  request<void>('POST', `/runs/${runId}/cancel`)

// Steps
export const listSteps = (runId: string, page = 1, limit = 100) =>
  request<PagedResult<StepListItem>>('GET', `/runs/${runId}/steps?page=${page}&limit=${limit}`)

export const markStepDone = (runId: string, step: string, output?: unknown) =>
  request<void>('POST', `/runs/${runId}/steps/${step}/done`, output ? { output } : {})

export const markStepFailed = (runId: string, step: string, error?: string) =>
  request<void>('POST', `/runs/${runId}/steps/${step}/failed`, error ? { error } : {})

export const retryStep = (runId: string, step: string) =>
  request<void>('POST', `/runs/${runId}/steps/${step}/retry`)

// Step output
export const getStepOutput = (runId: string, step: string) =>
  request<{ output: unknown; state: string }>('GET', `/runs/${runId}/steps/${step}`)

// Triggers
export const fireTrigger = (wfName: string, triggerIndex: number, idemKey?: string) =>
  request<{ run_id: string }>('POST', '/triggers/fire', {
    wf_name: wfName,
    trigger_index: triggerIndex,
    ...(idemKey ? { idem_key: idemKey } : {}),
  })

// Manifests
export const getWorkflowManifest = (name: string) =>
  request<{ manifest: string }>('GET', `/workflows/${encodeURIComponent(name)}/manifest`)
    .then(r => r.manifest)

// Events
export const listEvents = () =>
  request<string[]>('GET', '/events')

export const getEventSubscribers = (event: string) =>
  request<string[]>('GET', `/events/${event}/subscribers`)
