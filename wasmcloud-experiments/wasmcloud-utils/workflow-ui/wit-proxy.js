#!/usr/bin/env node
// wit-proxy.js — minimal Node.js sidecar for WIT extraction
// Usage: node workflow-ui/wit-proxy.js
// Listens on http://localhost:4444
// GET /api/wit?image=<oci-ref>

const http = require('node:http')
const { execFileSync } = require('node:child_process')
const { createHash } = require('node:crypto')
const { existsSync } = require('node:fs')
const path = require('node:path')
const os = require('node:os')

const PORT = 4444

function parseImports(witOutput) {
  const imports = []
  for (const line of witOutput.split('\n')) {
    const m = line.match(/^\s*import\s+([\w:-]+\/[\w-]+(?:\/[\w-]+)*)/)
    if (m) imports.push(m[1])
  }
  return imports
}

function witForFile(wasmPath) {
  const out = execFileSync('wasm-tools', ['component', 'wit', wasmPath], {
    timeout: 30_000,
    encoding: 'utf8',
  })
  return parseImports(out)
}

function handleRequest(req, res) {
  res.setHeader('Access-Control-Allow-Origin', '*')
  res.setHeader('Content-Type', 'application/json')

  if (req.method === 'OPTIONS') {
    res.writeHead(204)
    res.end()
    return
  }

  const url = new URL(req.url, `http://localhost:${PORT}`)
  if (url.pathname !== '/api/wit') {
    res.writeHead(404)
    res.end(JSON.stringify({ imports: [], error: 'Not found' }))
    return
  }

  const image = url.searchParams.get('image')
  if (!image) {
    res.writeHead(400)
    res.end(JSON.stringify({ imports: [], error: 'Missing image parameter' }))
    return
  }

  try {
    let wasmPath
    if (image.startsWith('file://')) {
      wasmPath = image.slice('file://'.length)
      if (!existsSync(wasmPath)) {
        res.writeHead(404)
        res.end(JSON.stringify({ imports: [], error: `File not found: ${wasmPath}` }))
        return
      }
    } else {
      const hash = createHash('sha256').update(image).digest('hex').slice(0, 16)
      wasmPath = path.join(os.tmpdir(), `wit-proxy-${hash}.wasm`)
      if (!existsSync(wasmPath)) {
        execFileSync('wash', ['pull', image, '--output', wasmPath], {
          timeout: 120_000,
          encoding: 'utf8',
        })
      }
    }

    const imports = witForFile(wasmPath)
    res.writeHead(200)
    res.end(JSON.stringify({ imports }))
  } catch (err) {
    const error = err instanceof Error ? err.message : String(err)
    res.writeHead(200) // return 200 so the client can show a graceful error
    res.end(JSON.stringify({ imports: [], error }))
  }
}

const server = http.createServer(handleRequest)
server.listen(PORT, '127.0.0.1', () => {
  console.log(`wit-proxy listening on http://localhost:${PORT}`)
})
