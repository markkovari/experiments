// API client for the vet-clinic backend. The frontend is served from the same
// origin as the API, so all paths are relative. The token lives in localStorage
// under `vet_token` and is sent as a Bearer header on every guarded call.

export const TOKEN_KEY = "vet_token"

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token)
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY)
}

export class ApiError extends Error {
  status: number
  data: unknown
  constructor(message: string, status: number, data: unknown) {
    super(message)
    this.name = "ApiError"
    this.status = status
    this.data = data
  }
}

type HttpMethod = "GET" | "POST" | "PUT" | "DELETE"

export async function api<T = unknown>(
  method: HttpMethod,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers["content-type"] = "application/json"
  const token = getToken()
  if (token) headers["authorization"] = `Bearer ${token}`

  const res = await fetch(path, {
    method,
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  const text = await res.text()
  const data = text ? (JSON.parse(text) as unknown) : null

  if (!res.ok) {
    const errMsg =
      (data && typeof data === "object" && "error" in data
        ? String((data as { error: unknown }).error)
        : null) ?? res.statusText
    throw new ApiError(errMsg, res.status, data)
  }
  return data as T
}

// ---- API contract types ----

export type Role = "pet-owner" | "doctor" | "admin"

export interface TokenPair {
  accessToken?: string
  // snake_case fallback tolerated
  access_token?: string
  refreshToken?: string
  expiresIn?: number | string
  sessionId?: string
}

export interface Me {
  subject: string
  tenant: string
  roles: string[]
  scopes: string[]
}

export interface RegisterResult {
  subject: string
  tenant: string
  role: string
}

export interface Pet {
  id: string
  name: string
  species: string
  owner: string
  notes?: string
}

export interface PetsResponse {
  pets: Pet[]
  viewer: string
}

export interface Appointment {
  id: string
  pet: string
  owner: string
  doctor: string
  datetime: string
  status: string
}

export interface AppointmentsResponse {
  appointments: Appointment[]
  viewer: string
}

export interface VisitNote {
  id: string
  appointment: string
  author: string
  text: string
  at: string
}

export interface NotesResponse {
  notes: VisitNote[]
}

export interface AuditEvent {
  id: string
  timestamp: string
  event: string
  outcome: string
  tenant: string
  subject: string
  detail: string
}

export interface AuditResponse {
  events: AuditEvent[]
}

export interface ValidationField {
  field: string
  code: string
  message: string
}

// Pull a usable access token out of a login response (snake fallback).
export function tokenFromPair(tp: TokenPair): string {
  const t = tp.accessToken ?? tp.access_token
  if (!t) throw new Error("login response had no access token")
  return t
}
