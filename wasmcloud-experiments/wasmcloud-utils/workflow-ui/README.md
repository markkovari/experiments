# workflow-ui

React + TypeScript SPA for the wasmcloud workflow engine.

## Stack

- **Vite** + React 18 + TypeScript
- **TanStack Query v5** — REST data fetching, cache invalidation via SSE events
- **Tailwind CSS v3**

## Prerequisites

- Node 18+
- `workflow-api` running on `http://localhost:8080`

## Development

```bash
npm install
npm run dev          # → http://localhost:5173
```

All `/api/*` requests are proxied to `http://localhost:8080` (Vite proxy config in `vite.config.ts`).

## Production build

```bash
npm run build        # output in dist/
npm run preview      # serve dist/ locally
```

## SSE live updates

The UI connects to `GET /api/sse` (proxied to `http://localhost:8080/sse`) via `EventSource`.
Events cause TanStack Query cache invalidations so run/step badges update without page reload.

Event types emitted by the backend:
| Type | Trigger |
|------|---------|
| `run.state` | run started, succeeded, failed, cancelled |
| `step.state` | step succeeded, failed, retried |

## Features

| Page | Description |
|------|-------------|
| **Workflows** | List, create (JSON editor), delete, start run |
| **Runs** | Filter by workflow + state, cancel, navigate to steps |
| **Steps** | Mark done (with output modal) / failed / retry; live badge updates |
| **Events** | List event names + subscriber functions |
