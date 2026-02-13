SHELL := /bin/bash
ROOT_DIR := $(shell pwd)
INFRA_DIR := $(ROOT_DIR)/infra
PIPELINE_DIR := $(ROOT_DIR)/pipeline

.PHONY: all infra-up infra-down build-a build-b ingest search stats clean

all: ingest

# Infrastructure
infra-up:
	docker compose -f $(INFRA_DIR)/docker-compose.yml up -d --build

infra-down:
	docker compose -f $(INFRA_DIR)/docker-compose.yml down

infra-reset:
	docker compose -f $(INFRA_DIR)/docker-compose.yml down -v
	docker compose -f $(INFRA_DIR)/docker-compose.yml up -d --build

# Wasmcloud builds (requires wash-cli and wasm32-wasip2 target)
build-a:
	cd $(ROOT_DIR)/counter-service-a && wash build

build-b:
	cd $(ROOT_DIR)/counter-service-b && wash build

build: build-a build-b

# Pipeline
setup:
	cd $(PIPELINE_DIR) && uv sync

ingest:
	$(ROOT_DIR)/ingest.sh

search: setup
	@read -p "Search query: " query; \
	cd $(PIPELINE_DIR) && uv run python -m src search "$$query"

stats: setup
	cd $(PIPELINE_DIR) && uv run python -m src stats

# Cleanup
clean:
	docker compose -f $(INFRA_DIR)/docker-compose.yml down -v
	rm -rf $(PIPELINE_DIR)/.venv
	rm -rf $(ROOT_DIR)/counter-service-a/build
	rm -rf $(ROOT_DIR)/counter-service-b/build
