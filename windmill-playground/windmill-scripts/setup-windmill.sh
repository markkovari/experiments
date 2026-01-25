#!/bin/bash

# Script to set up Windmill workspace and deploy factorial script
# Run this after docker-compose up

set -e

WINDMILL_URL="http://localhost:8001"
WORKSPACE="demo"
USERNAME="admin@windmill.dev"
PASSWORD="changeme"

echo "=== Windmill Setup Script ==="
echo ""

# Wait for Windmill to be ready
echo "Waiting for Windmill to be ready..."
until curl -s "${WINDMILL_URL}/api/version" > /dev/null; do
  echo "  Windmill not ready yet, waiting 5 seconds..."
  sleep 5
done
echo "✓ Windmill is ready!"
echo ""

# Login and get token
echo "Logging in to Windmill..."
TOKEN=$(curl -s -X POST "${WINDMILL_URL}/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"${USERNAME}\",\"password\":\"${PASSWORD}\"}" \
  | jq -r '.token // empty')

if [ -z "$TOKEN" ]; then
  echo "✗ Failed to login. Using default workspace setup."
  echo ""
  echo "Please complete setup manually:"
  echo "1. Go to ${WINDMILL_URL}"
  echo "2. Create account or login (default: admin@windmill.dev / changeme)"
  echo "3. Create a workspace named '${WORKSPACE}'"
  echo "4. Go to Scripts → +Script"
  echo "5. Choose TypeScript"
  echo "6. Copy content from windmill-scripts/factorial.ts"
  echo "7. Save as 'u/admin/factorial'"
  echo "8. Test the script with input: {\"number\": 5}"
  exit 1
fi

echo "✓ Logged in successfully!"
echo ""

# Check if workspace exists
echo "Checking workspace '${WORKSPACE}'..."
WORKSPACE_EXISTS=$(curl -s -X GET "${WINDMILL_URL}/api/w/${WORKSPACE}/workspaces" \
  -H "Authorization: Bearer ${TOKEN}" \
  -o /dev/null -w "%{http_code}")

if [ "$WORKSPACE_EXISTS" != "200" ]; then
  echo "  Workspace doesn't exist, would need to create it"
  echo "  Please create workspace '${WORKSPACE}' manually in the UI"
else
  echo "✓ Workspace '${WORKSPACE}' exists!"
fi

echo ""
echo "=== Manual Setup Required ==="
echo ""
echo "1. Open Windmill UI: ${WINDMILL_URL}"
echo "2. Login with: ${USERNAME} / ${PASSWORD}"
echo "3. Go to workspace: ${WORKSPACE}"
echo "4. Create a new TypeScript script:"
echo "   - Click 'Scripts' → '+ Script'"
echo "   - Select 'TypeScript (Deno)'"
echo "   - Name it: factorial"
echo "   - Path: u/admin/factorial"
echo "   - Copy the content from: windmill-scripts/factorial.ts"
echo "5. Test with input: {\"number\": 5, \"request_id\": \"test-1\"}"
echo ""
echo "Then update API_URL in docker-compose.yml to call Windmill"
echo ""
