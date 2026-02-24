#!/bin/bash
# Deploy the application to wasmCloud

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Deploying application to wasmCloud...${NC}"

# Wait for services to be ready
./scripts/wait-for-ready.sh

# Install wash if not available in the current context
if ! command -v wash &> /dev/null; then
    echo -e "${YELLOW}Installing wash CLI in container...${NC}"
    # This will run inside a container with wash already installed
fi

# Build the component if not already built
if [ ! -f "build/http_kv_counter_s.wasm" ]; then
    echo -e "${YELLOW}Building component...${NC}"
    docker compose run --rm builder
fi

# Deploy the application using wadm
echo -e "${YELLOW}Deploying wadm application...${NC}"

# Copy the built component to a location accessible by wasmCloud
# In a real setup, this would push to the OCI registry
# For now, we'll use wash app deploy directly

docker compose exec -T wasmcloud sh -c "
    # Install wash if not present
    if ! command -v wash &> /dev/null; then
        echo 'wash not found in wasmCloud container, using host wash'
        exit 1
    fi

    # Deploy the application
    wash app deploy /app/wadm.yaml
"

# Wait for deployment to complete
echo -e "${YELLOW}Waiting for application to be deployed...${NC}"
sleep 5

# Check if HTTP endpoint is responding
echo -e "${YELLOW}⏳ Waiting for HTTP endpoint...${NC}"
MAX_RETRIES=20
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:8080/ > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Application is deployed and responding!${NC}"
        exit 0
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}✗ Application failed to respond${NC}"
        echo "Showing wasmCloud logs:"
        docker compose logs wasmcloud
        exit 1
    fi
    echo "  Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 3
done
