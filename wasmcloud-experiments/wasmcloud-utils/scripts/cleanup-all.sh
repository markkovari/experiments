#!/usr/bin/env bash
set -euo pipefail

# Force cleanup all wasmCloud deployments

echo "🧹 Aggressive wasmCloud cleanup..."
echo ""

# Method 1: Drain all hosts (removes ALL components and providers)
echo "🌊 Draining all wasmCloud hosts..."
wash drain all --ctl-port 4222 2>/dev/null || true
sleep 2

# Method 2: Undeploy all apps
echo "🗑️  Undeploying all applications..."
apps=$(wash app list --output json 2>/dev/null | jq -r '.[].name' 2>/dev/null || echo "")

if [ -n "$apps" ]; then
    while IFS= read -r app; do
        if [ -n "$app" ] && [ "$app" != "null" ]; then
            echo "   - Undeploying: $app"
            wash app undeploy "$app" 2>/dev/null || true
        fi
    done <<< "$apps"
    sleep 2
fi

# Method 3: Delete NATS WADM manifests KV bucket (THE REAL FIX!)
echo "🔥 Deleting NATS WADM manifests KV bucket..."
nats kv rm wadm_manifests --server=nats://127.0.0.1:4222 --force 2>/dev/null || true
sleep 1

# Method 4: Stop wasmCloud host
echo "🛑 Stopping wasmCloud host..."
wash down 2>/dev/null || true
sleep 1

# Method 5: Kill all processes
echo "💀 Killing lingering processes..."
pkill -9 wash 2>/dev/null || true
pkill -9 wasmcloud 2>/dev/null || true
pkill -9 nats-server 2>/dev/null || true
sleep 2

# Method 6: Clean up NATS data and WADM state
echo "🧽 Cleaning NATS/WADM persistent state..."
rm -rf ~/.wash/downloads/nats-server/jetstream 2>/dev/null || true
rm -rf ~/.local/share/wasmcloud/data 2>/dev/null || true
rm -rf ~/.local/share/wash/downloads/wadm/data 2>/dev/null || true
rm -rf ~/.cache/wasmcloud 2>/dev/null || true

echo ""
echo "✅ Aggressive cleanup complete!"
echo ""
echo "Verification:"
echo "-------------"
if wash app list 2>/dev/null; then
    echo "⚠️  wasmCloud host still running (apps listed above)"
else
    echo "✅ No wasmCloud host running"
fi

echo ""
echo "To start fresh:"
echo "  wash up"
