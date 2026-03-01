// API client for the secrets-http wasm component.
//
// HTTP API (served by secrets_http.wasm via wasmCloud httpserver provider):
//   GET    /secrets              → string[]          (list names)
//   GET    /secrets/{name}       → SecretMetadata    (metadata only, no value)
//   POST   /secrets/{name}       → {ok:true}         body: {value: "<base64>"}
//   DELETE /secrets/{name}       → {ok:true}
//   POST   /secrets/{name}/rotate → SecretMetadata   body: {value: "<base64>"}
//
// The API never returns secret values — only metadata.

export interface SecretMetadata {
  name: string
  version: number
  created_at_ms: number
  updated_at_ms: number
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(path, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText)
    let msg = text
    try { msg = JSON.parse(text).error ?? text } catch { /* raw text */ }
    throw new Error(`${method} ${path} → ${res.status}: ${msg}`)
  }
  if (res.status === 204) return undefined as unknown as T
  return res.json() as Promise<T>
}

export const listSecrets = () =>
  request<string[]>('GET', '/secrets')

export const getSecretMeta = (name: string) =>
  request<SecretMetadata>('GET', `/secrets/${name}`)

export const setSecret = (name: string, valuePlaintext: string) => {
  const b64 = btoa(unescape(encodeURIComponent(valuePlaintext)))
  return request<{ ok: boolean }>('POST', `/secrets/${name}`, { value: b64 })
}

export const deleteSecret = (name: string) =>
  request<{ ok: boolean }>('DELETE', `/secrets/${name}`)

export const rotateSecret = (name: string, newValuePlaintext: string) => {
  const b64 = btoa(unescape(encodeURIComponent(newValuePlaintext)))
  return request<SecretMetadata>('POST', `/secrets/${name}/rotate`, { value: b64 })
}
