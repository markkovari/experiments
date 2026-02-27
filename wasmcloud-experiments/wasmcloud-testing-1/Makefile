.PHONY: help build test test-unit test-integration test-e2e clean run deploy undeploy dev

help: ## Show this help message
	@echo "Available targets:"
	@echo ""
	@grep -E '^[a-z][a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-25s %s\n", $$1, $$2}'

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
