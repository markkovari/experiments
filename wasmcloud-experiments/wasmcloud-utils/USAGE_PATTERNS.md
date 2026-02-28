# 🚀 Rate Limiter Usage Patterns

You now have **3 ways** to use these rate limiters in your wasmCloud applications!

## Pattern 1: HTTP Middleware (Transparent Proxy)

**When to use:** You want a drop-in rate limiter in front of your HTTP API

**How it works:** Rate limiter acts as an HTTP proxy - checks limits then forwards requests

```
Client → [Rate Limiter w/ HTTP] → Your App
```

### Deploy

```bash
wash app deploy wadm/pattern1-http-middleware.yaml
```

### Test

```bash
# First request - should succeed
curl -H "X-User-Id: alice" http://localhost:8080/api
# {"status":"ok","user_id":"alice","tokens_remaining":9}

# Spam requests - will eventually hit limit
for i in {1..15}; do
  curl -H "X-User-Id: alice" http://localhost:8080/api
done
# {"error":"rate_limit_exceeded","retry_after_seconds":1}
```

### When to Use

✅ Want zero changes to existing app code
✅ Need rate limiting for HTTP APIs
✅ Want centralized rate limiting
✅ Multiple apps share one rate limiter

❌ Non-HTTP protocols
❌ Need custom rate limit logic per endpoint

---

## Pattern 2: Library Import (Embedded)

**When to use:** You want rate limiting logic inside your component

**How it works:** Your component imports the rate limiter interface and uses it directly

```
Your App Component
  ├─ HTTP Handler
  ├─ Business Logic
  └─ Rate Limiter (imported)
```

### Example Code

```rust
// In your component
wit_bindgen::generate!({
    world: "consumer-with-ratelimit",
    path: "../wit",
});

impl Guest for MyApp {
    fn handle(request: Request) -> Response {
        // Import and use rate limiter
        let rate_request = RateLimitRequest {
            user_id: extract_user(&request),
            tokens_requested: 1,
            timestamp_ms: now(),
        };

        match rate_limiter::check_rate_limit(rate_request) {
            Ok(resp) if resp.allowed => handle_request(request),
            Ok(resp) => rate_limited_response(resp.retry_after_ms),
            Err(e) => error_response(e),
        }
    }
}
```

### Deploy

```bash
# Build your app with rate limiter imported
cargo build --package consumer-app --target wasm32-wasip1 --release

# Deploy
wash app deploy wadm/pattern2-library-import.yaml
```

### When to Use

✅ Want fine-grained control over rate limiting
✅ Different limits for different endpoints
✅ Custom rate limit logic
✅ Minimal network overhead (no IPC)

❌ Want to share rate limiter across apps
❌ Don't want to recompile when changing limits

---

## Pattern 3: Sidecar (Microservices)

**When to use:** You have multiple apps sharing a rate limiter via messaging

**How it works:** Your app sends NATS messages to a rate limiter sidecar

```
App 1 ──┐
App 2 ──┼──> NATS ──> [Rate Limiter Sidecar]
App 3 ──┘              └─> NATS KV (state)
```

### Example Flow

```rust
// In your app
async fn handle_request(req: Request) -> Response {
    // Send rate limit check via NATS
    let msg = nats::Message {
        subject: "ratelimit.check",
        payload: json!({
            "user_id": extract_user(&req),
            "tokens_requested": 1
        })
    };

    let response = nats::request(msg).await?;
    let rate_resp: RateLimitResponse = serde_json::from_slice(&response.payload)?;

    if rate_resp.allowed {
        handle_business_logic(req)
    } else {
        rate_limited_response(rate_resp.retry_after_ms)
    }
}
```

### Deploy

```bash
wash app deploy wadm/pattern3-sidecar.yaml
```

### When to Use

✅ Multiple apps need rate limiting
✅ Want independent scaling (app vs limiter)
✅ Shared state across apps
✅ Polyglot - any app can use it
✅ Zero code coupling

❌ Need lowest latency (adds network hop)
❌ Simple single-app scenario

---

## Comparison Table

| Feature | HTTP Middleware | Library Import | Sidecar |
|---------|----------------|----------------|---------|
| **Setup Complexity** | ⭐ Easy | ⭐⭐ Medium | ⭐⭐⭐ Complex |
| **Performance** | ⭐⭐ Fast | ⭐⭐⭐ Fastest | ⭐ Network overhead |
| **Flexibility** | ⭐ Limited | ⭐⭐⭐ Very flexible | ⭐⭐ Flexible |
| **Code Changes** | ✅ None | ❌ Requires imports | ⭐⭐ NATS integration |
| **Shared State** | ✅ Yes | ❌ Per-component | ✅ Yes |
| **Multi-App** | ✅ Yes | ❌ No | ✅ Yes |
| **Protocols** | HTTP only | Any | Any |

---

## Quick Start Examples

### Pattern 1: Just Deploy It

```bash
# Single command - you're done!
wash app deploy wadm/pattern1-http-middleware.yaml

# Test it
curl -H "X-User-Id: test" http://localhost:8080/anything
```

### Pattern 2: Add to Your Code

```toml
# Cargo.toml
[dependencies]
wit-bindgen = "0.39"

# In your WIT
world my-app {
    import wasmcloud:ratelimit/rate-limiter@0.1.0;
    export wasi:http/incoming-handler;
}
```

```rust
// src/lib.rs
use wasmcloud::ratelimit::rate_limiter;

// Use it anywhere in your code!
if !rate_limiter::check_rate_limit(req)?.allowed {
    return rate_limited();
}
```

### Pattern 3: NATS Messaging

```rust
// Send check request
nats.publish("ratelimit.check", user_data).await?;

// Rate limiter responds
let response = nats.subscribe("ratelimit.response").await?;
```

---

## Which Pattern Should I Use?

### Use **HTTP Middleware** if:
- ✅ "I just want rate limiting on my API NOW"
- ✅ "I don't want to change any code"
- ✅ "I have multiple HTTP services to protect"

### Use **Library Import** if:
- ✅ "I need custom logic per endpoint"
- ✅ "I want the absolute best performance"
- ✅ "My rate limiting is app-specific"

### Use **Sidecar** if:
- ✅ "I have many microservices"
- ✅ "I want independent scaling"
- ✅ "I'm using multiple languages/components"
- ✅ "I want centralized rate limit management"

---

## Mix and Match!

You can use different patterns for different services:

```yaml
# API Gateway - HTTP Middleware
- /api/public → Pattern 1 (HTTP Middleware)

# Core Service - Library Import
- /api/internal → Pattern 2 (Embedded)

# Microservices - Sidecar
- service-a, service-b, service-c → Pattern 3 (Sidecar)
```

---

## Next Steps

1. **Try Pattern 1** - Easiest to get started
   ```bash
   ./scripts/build-components.sh
   wash up
   wash app deploy wadm/pattern1-http-middleware.yaml
   ```

2. **Learn Pattern 2** - Best performance
   - Check `examples/consumer-app/` for full example
   - Add rate limiter import to your WIT
   - Call `rate_limiter::check_rate_limit()` in your code

3. **Scale with Pattern 3** - Production microservices
   - Set up NATS messaging in your apps
   - Deploy shared rate limiter sidecar
   - Scale apps and limiter independently

---

## 🎯 The Answer to Your Question

> "Can I just use this and never write a rate limiter again?"

**YES!** Here's how:

- **Quick API?** → Pattern 1 (5 minutes)
- **Custom app?** → Pattern 2 (import it)
- **Enterprise?** → Pattern 3 (sidecar)

All 3 patterns give you production-ready rate limiting with:
- ✅ Token Bucket (burst traffic)
- ✅ Leaky Bucket (smooth flow)
- ✅ Sliding Window (time-based quotas)
- ✅ NATS KV persistence
- ✅ Per-user tracking
- ✅ Zero external dependencies

**You literally never need to implement rate limiting logic again!** 🎉
