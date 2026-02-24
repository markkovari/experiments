#!/bin/bash
# Simple deployment script for HTTP KV Counter
# Uses wash 0.43.0 for deployment

set -e

WASH=/tmp/wash-old
NATS_URL=${NATS_URL:-nats://localhost:4222}

echo "🚀 Deploying HTTP KV Counter"
echo "================================"

# Start providers
echo ""
echo "1. Starting HTTP Server provider..."
$WASH_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH start provider ghcr.io/wasmcloud/http-server:0.23.1 httpserver

echo ""
echo "2. Starting NATS KV provider..."
WASMCLOUD_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH start provider ghcr.io/wasmcloud/keyvalue-nats:0.3.1 keyvalue-nats

echo ""
echo "3. Starting component..."
WASMCLOUD_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH start component file://./build/http_kv_counter_s.wasm http-kv-counter

# Wait for everything to start
sleep 2

echo ""
echo "4. Creating links..."
WASMCLOUD_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH link put http-kv-counter httpserver wasi http --interface incoming-handler

WASMCLOUD_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH link put http-kv-counter keyvalue-nats wasi keyvalue --interface store

WASMCLOUD_CTL_HOST=localhost WASMCLOUD_CTL_PORT=4222 $WASH link put http-kv-counter keyvalue-nats wasi keyvalue --interface atomics

echo ""
echo "✅ Deployment complete!"
echo ""
echo "Note: HTTP server may need manual configuration for address (0.0.0.0:8080)"
echo "Check logs at ~/.local/share/wash/downloads/wasmcloud.log"
