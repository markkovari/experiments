import { useEffect } from 'react'
import type { QueryClient } from '@tanstack/react-query'

export interface SseEvent {
  type: 'run.state' | 'step.state'
  run_id: string
  wf_name?: string
  step?: string
  state: string
  ts_ms: number
  seq: number
}

export function useWorkflowEvents(queryClient: QueryClient) {
  useEffect(() => {
    const es = new EventSource('/api/sse')

    es.onmessage = (e) => {
      try {
        const ev: SseEvent = JSON.parse(e.data as string)
        if (ev.type === 'run.state') {
          queryClient.invalidateQueries({ queryKey: ['runs'] })
          queryClient.invalidateQueries({ queryKey: ['run', ev.run_id] })
        }
        if (ev.type === 'step.state') {
          queryClient.invalidateQueries({ queryKey: ['steps', ev.run_id] })
        }
      } catch {
        // ignore malformed events
      }
    }

    es.onerror = () => {
      // EventSource reconnects automatically
    }

    return () => es.close()
  }, [queryClient])
}
