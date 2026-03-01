import { useQuery } from '@tanstack/react-query'
import { inferProviders, type ProviderSuggestion } from '../lib/providerCatalog'

function isValidImageRef(image: string): boolean {
  if (!image || image.trim().length === 0) return false
  if (image.startsWith('file://')) return true
  // must look like registry/repo:tag or registry/repo@sha
  return /^[\w.-]+\/[\w./-]+(:\w[\w.-]*|@sha256:[a-f0-9]{64})?$/.test(image)
}

async function fetchWitImports(image: string): Promise<string[]> {
  const res = await fetch(`/api/wit?image=${encodeURIComponent(image)}`)
  const json = await res.json() as { imports: string[]; error?: string }
  if (json.error) throw new Error(json.error)
  return json.imports
}

export function useWitInference(ociImage: string): {
  imports: string[]
  suggestions: ProviderSuggestion[]
  isLoading: boolean
  error: string | null
} {
  const enabled = isValidImageRef(ociImage)

  const { data, isLoading, error } = useQuery({
    queryKey: ['wit', ociImage],
    queryFn: () => fetchWitImports(ociImage),
    enabled,
    staleTime: 5 * 60 * 1000,
    retry: false,
  })

  const imports = data ?? []
  return {
    imports,
    suggestions: inferProviders(imports),
    isLoading: enabled && isLoading,
    error: error ? (error as Error).message : null,
  }
}
