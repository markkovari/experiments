#!/bin/bash

# Quick script to enable Windmill mode
# This will guide you through getting a token and updating the configuration

set -e

WINDMILL_URL="http://localhost:8001"
COMPOSE_FILE="docker-compose.yml"

echo "================================"
echo "  Enable Windmill Mode"
echo "================================"
echo ""

# Check if services are running
echo "Checking if services are running..."
if ! curl -s "${WINDMILL_URL}/api/version" > /dev/null 2>&1; then
    echo "❌ Windmill is not running!"
    echo ""
    echo "Please start services first:"
    echo "  docker-compose up -d"
    echo ""
    exit 1
fi
echo "✅ Windmill is running"
echo ""

# Instructions for getting token
echo "================================"
echo "  Step 1: Create Windmill Token"
echo "================================"
echo ""
echo "1. Open Windmill UI: ${WINDMILL_URL}"
echo "2. Complete initial setup (create admin user if needed)"
echo "3. Create or use workspace: 'demo'"
echo "4. Go to Account Settings → Tokens"
echo "5. Click 'New Token'"
echo "   - Name: api-token"
echo "   - Expiration: Never"
echo "6. Copy the token"
echo ""
read -p "Press Enter once you have your token ready..."
echo ""
read -sp "Paste your Windmill token: " WINDMILL_TOKEN
echo ""
echo ""

if [ -z "$WINDMILL_TOKEN" ]; then
    echo "❌ No token provided"
    exit 1
fi

echo "✅ Token received"
echo ""

# Instructions for deploying script
echo "================================"
echo "  Step 2: Deploy Factorial Script"
echo "================================"
echo ""
echo "1. In Windmill UI, go to Scripts → + Script"
echo "2. Choose 'TypeScript (Deno)'"
echo "3. Set path: u/admin/factorial"
echo "4. Copy content from: windmill-scripts/factorial.ts"
echo "5. Click Save"
echo "6. Test with: {\"number\": 5, \"request_id\": \"test-1\"}"
echo ""
read -p "Press Enter once script is deployed and tested..."
echo ""

# Update docker-compose.yml
echo "================================"
echo "  Step 3: Update Configuration"
echo "================================"
echo ""
echo "Updating docker-compose.yml..."

# Backup original
cp "${COMPOSE_FILE}" "${COMPOSE_FILE}.backup"
echo "  Created backup: ${COMPOSE_FILE}.backup"

# Update USE_WINDMILL and WINDMILL_TOKEN
sed -i.tmp "s|USE_WINDMILL: \"false\"|USE_WINDMILL: \"true\"|g" "${COMPOSE_FILE}"
sed -i.tmp "s|WINDMILL_TOKEN: \"\"|WINDMILL_TOKEN: \"${WINDMILL_TOKEN}\"|g" "${COMPOSE_FILE}"
rm -f "${COMPOSE_FILE}.tmp"

echo "  Updated USE_WINDMILL: true"
echo "  Updated WINDMILL_TOKEN: ${WINDMILL_TOKEN:0:10}..."
echo ""
echo "✅ Configuration updated"
echo ""

# Restart API
echo "================================"
echo "  Step 4: Restart API Service"
echo "================================"
echo ""
echo "Restarting API service..."
docker-compose restart api
echo ""
echo "✅ API restarted"
echo ""

# Check API logs
echo "Checking API logs..."
sleep 2
docker-compose logs --tail=5 api | grep -i windmill || echo "  (no windmill logs yet)"
echo ""

# Final instructions
echo "================================"
echo "  ✅ Windmill Mode Enabled!"
echo "================================"
echo ""
echo "You can now:"
echo "  1. Test via frontend: http://localhost:3000"
echo "  2. View runs in Windmill: ${WINDMILL_URL}/runs"
echo ""
echo "To disable Windmill mode:"
echo "  1. Edit docker-compose.yml"
echo "  2. Set USE_WINDMILL: \"false\""
echo "  3. Run: docker-compose restart api"
echo ""
