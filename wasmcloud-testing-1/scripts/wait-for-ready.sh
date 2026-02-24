#!/bin/bash
# Wait for wasmCloud services to be ready

set -e

echo "Waiting for services to be ready..."

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Wait for NATS
echo -e "${YELLOW}⏳ Waiting for NATS server...${NC}"
MAX_RETRIES=30
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:8222/healthz > /dev/null 2>&1; then
        echo -e "${GREEN}✓ NATS is ready${NC}"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}✗ NATS failed to start${NC}"
        exit 1
    fi
    echo "  Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 2
done

# Wait for NATS JetStream
echo -e "${YELLOW}⏳ Waiting for NATS JetStream...${NC}"
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:8222/jsz > /dev/null 2>&1; then
        echo -e "${GREEN}✓ JetStream is ready${NC}"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}✗ JetStream failed to start${NC}"
        exit 1
    fi
    echo "  Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 2
done

# Wait for wasmCloud host
echo -e "${YELLOW}⏳ Waiting for wasmCloud host...${NC}"
RETRY_COUNT=0
while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -s http://localhost:4000/ > /dev/null 2>&1; then
        echo -e "${GREEN}✓ wasmCloud host is ready${NC}"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
        echo -e "${RED}✗ wasmCloud host failed to start${NC}"
        exit 1
    fi
    echo "  Attempt $RETRY_COUNT/$MAX_RETRIES..."
    sleep 2
done

echo -e "${GREEN}✓ All services are ready!${NC}"
