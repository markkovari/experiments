# EDA with NATS, AsyncAPI, and EventCatalog

This project implements a robust, discoverable, and governed event-driven architecture.

## Components

1.  **NATS**: Messaging backbone (JetStream).
2.  **AsyncAPI**: Technical specifications in `api/`.
3.  **EventCatalog**: Visual documentation (configured in `infra/`).
4.  **Golang**: Implementation in `cmd/` and `internal/`.

## Getting Started

### 1. Start Infrastructure

```bash
cd infra
docker-compose up -d
```

### 2. Run the Go Service

This will publish an `orders.created` event to NATS.

```bash
go run cmd/order-service/main.go
```

### 3. Documentation (EventCatalog)

To generate the documentation from AsyncAPI:

1. Install EventCatalog CLI: `npm install -g @eventcatalog/cli`
2. Initialize EventCatalog (if not already done): `npx @eventcatalog/create-eventcatalog@latest eventcatalog`
3. Use the AsyncAPI plugin to import `api/order-service.yaml`.

## Project Structure

- `api/`: AsyncAPI specifications.
- `cmd/`: Service implementations.
- `infra/`: Docker Compose and infrastructure config.
- `internal/`: Shared library code (NATS handlers, etc.).
