.PHONY: help build test test-unit test-integration test-e2e clean run deploy undeploy dev \
        docker-build docker-up docker-down docker-test docker-logs docker-clean \
        docker-build-only docker-deploy docker-run docker-test-all

help: ## Show this help message
	@echo "Available targets:"
	@echo ""
	@echo "🐳 Docker Commands (no local dependencies required):"
	@grep -E '^docker-[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "🔧 Local Development Commands (requires wash + NATS installed):"
	@grep -E '^[a-z][a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v '^docker-' | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-25s %s\n", $$1, $$2}'

build: ## Build the WebAssembly component
	wash build

test: ## Run all tests (unit + integration)
	cargo test --lib
	cargo test --test integration_test

test-unit: ## Run unit tests only
	cargo test --lib

test-integration: ## Run integration tests only
	cargo test --test integration_test

test-e2e: ## Run end-to-end tests (requires running wasmCloud and NATS)
	cargo test --test e2e_test -- --ignored

clean: ## Clean build artifacts
	cargo clean
	rm -rf build/
	rm -rf wit/deps/

nats: ## Start NATS server with JetStream
	nats-server -js

wash-up: ## Start wasmCloud host
	wash up

deploy: build ## Deploy the application to wasmCloud
	wash app deploy wadm.yaml

undeploy: ## Undeploy the application from wasmCloud
	wash app undeploy http-kv-counter

dev: ## Run in development mode with hot reload
	wash dev

run: nats wash-up deploy ## Start everything (NATS + wasmCloud + deploy)

stop: undeploy ## Stop everything
	wash down
	pkill nats-server

logs: ## Show application logs
	wash app logs http-kv-counter

status: ## Show application status
	wash app list
	wash get hosts
	wash get inventory

check: ## Check the project for errors
	cargo check --target wasm32-wasip2

fmt: ## Format the code
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy --target wasm32-wasip2

install-deps: ## Install required dependencies
	rustup target add wasm32-wasip2
	cargo install wash-cli || true

# ============================================================================
# Docker-based commands (no local NATS/wash installation required)
# ============================================================================

docker-build-only: ## Build the component using Docker
	docker compose -f docker-compose.yml -f docker-compose.test.yml run --rm builder

docker-up: ## Start all services (NATS + wasmCloud) using Docker
	docker compose up -d
	@echo "Waiting for services to be ready..."
	@sleep 5
	@./scripts/wait-for-ready.sh

docker-down: ## Stop all Docker services
	docker compose down

docker-deploy: docker-build-only ## Build and deploy the application to Docker wasmCloud
	@echo "Deploying application..."
	docker compose -f docker-compose.yml -f docker-compose.test.yml run --rm deployer

docker-run: docker-up docker-deploy ## Start everything with Docker (build + deploy + run)
	@echo "✓ Application is running at http://localhost:8081"
	@echo "Run 'make docker-logs' to see logs"
	@echo "Run 'make docker-test' to run tests"
	@echo "Run 'make docker-down' to stop"

docker-test: ## Run all tests using Docker
	./scripts/run-tests.sh all

docker-test-e2e: ## Run e2e tests using Docker
	./scripts/run-tests.sh e2e

docker-logs: ## Show Docker container logs
	docker compose logs -f

docker-logs-wasmcloud: ## Show only wasmCloud logs
	docker compose logs -f wasmcloud

docker-logs-nats: ## Show only NATS logs
	docker compose logs -f nats

docker-clean: docker-down ## Clean all Docker resources
	docker compose down -v
	docker system prune -f
	rm -rf build/ target/

docker-shell-wasmcloud: ## Open shell in wasmCloud container
	docker compose exec wasmcloud sh

docker-shell-nats: ## Open shell in NATS container
	docker compose exec nats sh

docker-restart: docker-down docker-up ## Restart all Docker services

docker-build: docker-build-only ## Alias for docker-build-only

docker-test-all: docker-up docker-deploy docker-test-e2e ## Run complete test suite with Docker
