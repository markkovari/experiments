export type ProviderSuggestion = {
  id: string
  label: string
  image: string
  namespace: string
  pkg: string
  interfaces: string[]
  triggeredBy: string[]
}

export const PROVIDER_CATALOG: ProviderSuggestion[] = [
  {
    id: 'keyvalue-nats',
    label: 'Add KV provider',
    image: 'ghcr.io/wasmcloud/keyvalue-nats:0.3.1',
    namespace: 'wasi',
    pkg: 'keyvalue',
    interfaces: ['store', 'atomics'],
    triggeredBy: ['wasi:keyvalue/'],
  },
  {
    id: 'http-client',
    label: 'Add HTTP client provider',
    image: 'ghcr.io/wasmcloud/http-client:0.12.0',
    namespace: 'wasi',
    pkg: 'http',
    interfaces: ['outgoing-handler'],
    triggeredBy: ['wasi:http/outgoing-handler'],
  },
  {
    id: 'messaging-nats',
    label: 'Add NATS messaging provider',
    image: 'ghcr.io/wasmcloud/messaging-nats:0.23.1',
    namespace: 'wasi',
    pkg: 'messaging',
    interfaces: ['consumer', 'producer'],
    triggeredBy: ['wasi:messaging/'],
  },
  {
    id: 'secrets-component',
    label: 'Add secrets provider',
    image: 'file://../target/wasm32-wasip1/release/secrets_component.wasm',
    namespace: 'wasmcloud',
    pkg: 'secrets',
    interfaces: ['secret-store'],
    triggeredBy: ['wasmcloud:secrets/'],
  },
]

export function inferProviders(imports: string[]): ProviderSuggestion[] {
  return PROVIDER_CATALOG.filter(p =>
    imports.some(imp => p.triggeredBy.some(prefix => imp.startsWith(prefix)))
  )
}
