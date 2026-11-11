#!/usr/bin/env bash
# Build, start the server on a throwaway SQLite file, run all Hurl tests, stop the server.
set -euo pipefail
cd "$(dirname "$0")"
cargo build -q
dir=$(mktemp -d)
port=${PORT:-3917}
DATABASE_URL="sqlite:$dir/test.db" ADDR="127.0.0.1:$port" ./target/debug/concurrent-http-req >"$dir/server.log" 2>&1 &
pid=$!
trap 'kill $pid 2>/dev/null; rm -rf "$dir"' EXIT
for _ in $(seq 50); do curl -s "127.0.0.1:$port" >/dev/null && break; sleep 0.1; done
hurl --test --variable host="http://127.0.0.1:$port" tests/*.hurl
