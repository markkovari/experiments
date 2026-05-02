.PHONY: up down stress catalog lint proto

# Start everything
up:
	cd infra && docker-compose up -d --build

# Stop everything
down:
	cd infra && docker-compose down

# Run the 10-minute stress test
stress:
	./stress_test.sh

# Start the EventCatalog dev server
catalog:
	cd eventcatalog && PROJECT_DIR=.. npm run dev

# Run AsyncAPI Linting
lint:
	cd eventcatalog && npm run lint

# Generate all code from Proto files
proto:
	buf generate --path proto
	buf generate --template buf.gen.go.yaml --path proto

# View real-time logs
logs:
	cd infra && docker-compose logs -f
