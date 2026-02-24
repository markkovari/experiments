# Testing Guide for HTTP KV Counter

## Overview

This project uses a **3-tier testing strategy**:
1. **Unit Tests** - Fast, no runtime required
2. **Integration Tests** - Business logic testing
3. **E2E Tests** - Full stack testing with wasmCloud

---

## 1. Unit Tests (Fastest - ~0.1s)

Tests pure Rust logic without any wasmCloud runtime.

```bash
cargo test --lib
```

**What it tests:**
- Path parsing (`/`, `/counter`, `/invalid/path`)
- JSON serialization/deserialization
- Data structures

**Coverage:** ✅ 5/5 tests passing

---

## 2. Integration Tests (Fast - ~0.5s)

Tests business logic without full wasmCloud runtime.

```bash
cargo test --test integration_test
```

**What it tests:**
- HTTP routing logic
- Counter increment operations
- JSON response formatting
- Edge cases (empty strings, special characters)
- Concurrent operations logic

**Coverage:** ✅ 9/9 tests passing

---

## 3. E2E Tests (Full Stack)

Tests the complete system with wasmCloud, NATS, and providers.

### Option A: Automated E2E Tests (Requires Full Setup)

```bash
# 1. Start infrastructure
make docker-up

# 2. Deploy component (manual step - see DEPLOYMENT.md)
# wash app deploy wadm.yaml

# 3. Run automated tests
cargo test --test e2e_test -- --ignored
```

**What it tests:**
- Actual HTTP requests to deployed component
- NATS KV provider integration
- 3-second TTL expiration
- Full request/response cycle

**Note:** Currently requires wasmCloud 1.x deployment tooling.

### Option B: Manual E2E Testing (Recommended for now)

```bash
# 1. Start Docker stack
make docker-up

# 2. Deploy component (when deployment works)

# 3. Run manual test script
./scripts/manual-e2e-test.sh 8081
```

**The script tests:**
1. POST /counter1 → Create counter
2. GET /counter1 → Read counter
3. POST /counter1 → Increment counter
4. POST /counter2 → Create another counter
5. GET / → List all counters
6. Wait 4s → Verify TTL expiration

---

## Testing Strategy Summary

```bash
# Quick feedback loop (no runtime)
cargo test --lib && cargo test --test integration_test

# Full E2E when infrastructure is ready
./scripts/manual-e2e-test.sh
```

### Test Coverage

| Level | Tests | Speed | Runtime Required |
|-------|-------|-------|-----------------|
| Unit | 5 | ~0.1s | ❌ None |
| Integration | 9 | ~0.5s | ❌ None |
| E2E | 9 | ~10s | ✅ wasmCloud + NATS + Providers |

---

## CI/CD Pipeline

```yaml
# Example GitHub Actions
- name: Unit Tests
  run: cargo test --lib

- name: Integration Tests
  run: cargo test --test integration_test

- name: E2E Tests
  run: |
    make docker-up
    # Deploy step here
    ./scripts/manual-e2e-test.sh
```

---

## Troubleshooting

### E2E Tests Failing?

1. **Check services are running:**
   ```bash
   docker compose ps
   ```

2. **Check component is deployed:**
   ```bash
   curl http://localhost:8081/test
   ```

3. **Check logs:**
   ```bash
   make docker-logs-wasmcloud
   ```

### Deployment Issues?

- **wash 2.0-rc.7:** Deployment commands still evolving
- **wash 1.x:** Use `wash app deploy wadm.yaml`
- **Docker:** Component built successfully at `build/http_kv_counter_s.wasm`

---

## Best Practices

1. **Always run unit + integration tests first** - Fast feedback
2. **Run E2E tests before commits** - Catch integration issues
3. **Use manual E2E script** until wash 2.0 stable
4. **Test TTL behavior** - Critical requirement (3-second expiration)

---

## What's Working Right Now

✅ **Component builds successfully** (`build/http_kv_counter_s.wasm` - 124KB)
✅ **Unit tests pass** (5/5) - ~0.1s
✅ **Integration tests pass** (9/9) - ~0.5s
✅ **Code implements all requirements**
✅ **wadm.yaml validated** (fixed replicas string issue at wadm.yaml:70)
⚠️ **E2E deployment** - wasmCloud tooling in transition (1.x → 2.0)

### Test Results (Latest Run)
```bash
$ cargo test --lib && cargo test --test integration_test

running 5 tests
test tests::test_parse_path_root ... ok
test tests::test_parse_path_invalid ... ok
test tests::test_parse_path_counter ... ok
test tests::test_counter_data_deserialization ... ok
test tests::test_counter_data_serialization ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 9 tests
test test_counter_increment ... ok
test test_concurrent_increments ... ok
test test_path_parsing ... ok
test test_json_array_serialization ... ok
test test_http_routing_logic ... ok
test test_edge_cases ... ok
test test_error_response_format ... ok
test test_json_serialization ... ok
test test_value_parsing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Total: 14/14 tests passing** ✅
