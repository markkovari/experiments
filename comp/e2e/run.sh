#!/usr/bin/env bash
# End to end: real manifests, real hosts, real requests.
#
# Everything else in `bench/` measures one property at a time on a stack built for
# that measurement. This deploys SIX apps to ONE fleet at once and asserts what each
# should do — three that must serve traffic, three that must be refused with a reason
# a human can act on. Deploying them together is the point: a refusal that also stops
# the other five apps from being placed is a bug this catches and a single-app test
# cannot.
#
# Fast on purpose (~1 minute), so it can be run before pushing rather than admired in
# a document.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)/comp"
SP=${SP:-$(mktemp -d)}
R=components/target/wasm32-wasip2/release
PIDS=()
trap 'for p in "${PIDS[@]}"; do kill "$p" 2>/dev/null; done; sleep 1' EXIT

for f in "$R/gate_domain.wasm" "$R/record_store.wasm" "$R/shaper.wasm" \
         components/target/gate_domain.composed.wasm; do
  [ -f "$f" ] || { echo "missing $f — run \`just build\` and \`just compose-gate\`"; exit 1; }
done

mkdir -p "$SP/nats"
nats-server -js -sd "$SP/nats" -a 127.0.0.1 -p 4232 >"$SP/nats.log" 2>&1 & PIDS+=($!)
# The fused app gets the composed artifact; the linked one gets the three raw
# components, which is the difference between the two strategies (ADR-0005).
python3 bench/adversarial/stub-control-plane.py e2e/fixtures.json \
  "{\"gate\":\"components/target/gate_domain.composed.wasm\",\"record-store\":\"$R/record_store.wasm\",\"shaper\":\"$R/shaper.wasm\"}" \
  8099 >"$SP/plat.log" 2>&1 & PIDS+=($!)
sleep 2

for n in 1 2; do
  mkdir -p "$SP/n$n"
  ./host/target/release/comp-host --lattice-nats nats://127.0.0.1:4232 --node "n$n" \
    --lattice e2e --addr "127.0.0.1:390$n" --advertise-addr "127.0.0.1:390$n" \
    --state-dir "$SP/n$n" >"$SP/n$n.log" 2>&1 & PIDS+=($!)
done
sleep 2
./reconciler/target/release/comp-reconciler --platform-url http://127.0.0.1:8099 \
  --secret test-secret --nats-url nats://127.0.0.1:4232 --lattice e2e --interval 3 \
  >"$SP/rec.log" 2>&1 & PIDS+=($!)
./reconciler/target/release/comp-ingress --addr 127.0.0.1:8094 \
  --nats-url nats://127.0.0.1:4232 --lattice e2e --refresh-secs 2 >"$SP/ingress.log" 2>&1 & PIDS+=($!)

echo "  deploying six apps to one fleet..."
sleep 30
python3 e2e/check.py "$SP"
