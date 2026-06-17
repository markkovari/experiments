# JobRunner — React + Rust over Connect-RPC.
#
# One .proto drives both sides: the Rust server generates types via
# connectrpc-build in server/build.rs (on `cargo build`); the web client
# generates TypeScript via `buf generate` into web/src/gen.

PROTO_DIR := proto
SERVER_DIR := server
WEB_DIR := web
# buf needs the protoc-gen-es plugin, installed under web/node_modules/.bin.
BUF_PATH := $(CURDIR)/$(WEB_DIR)/node_modules/.bin

.PHONY: gen install server web dev clean help

help:
	@echo "make install  - install web deps (npm) "
	@echo "make gen      - regenerate TypeScript client from proto (buf)"
	@echo "make server   - run the Rust Connect server (:8088)"
	@echo "make web       - run the Vite dev server (:5173, proxies to :8088)"
	@echo "make dev      - run server in background + web in foreground"
	@echo "make clean    - cargo clean + remove generated TS"

install:
	cd $(WEB_DIR) && npm install

# Rust regenerates on `cargo build` via build.rs; this target does the TS side.
gen:
	cd $(PROTO_DIR) && PATH="$(BUF_PATH):$$PATH" buf generate

server:
	cd $(SERVER_DIR) && cargo run

web:
	cd $(WEB_DIR) && npm run dev

# Convenience: start the backend in the background, then the frontend.
# Ctrl-C stops Vite; the trap stops the backend too.
dev:
	cd $(SERVER_DIR) && cargo run & \
	SERVER_PID=$$!; \
	trap "kill $$SERVER_PID 2>/dev/null" EXIT INT TERM; \
	cd $(WEB_DIR) && npm run dev

clean:
	cd $(SERVER_DIR) && cargo clean
	rm -rf $(WEB_DIR)/src/gen $(WEB_DIR)/dist
