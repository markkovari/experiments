# Windmill Setup Guide

This guide explains how to enable Windmill orchestration for the distributed factorial calculator.

## Architecture Modes

The system supports two modes:

### 1. Direct NATS Mode (Default - `USE_WINDMILL=false`)
- API → Orchestrator → NATS Workers
- Distributed recursion via NATS request-response
- No Windmill jobs visible in UI
- ✅ Currently working out of the box

### 2. Windmill Orchestration Mode (`USE_WINDMILL=true`)
- API → Windmill Job → NATS Workers
- Each factorial calculation is a Windmill job
- **View all runs in Windmill UI at http://localhost:8001/runs**
- 📋 Requires manual setup (steps below)

## Enabling Windmill Mode

### Step 1: Start the Services

```bash
docker-compose up -d
```

Wait ~30 seconds for all services to be ready.

### Step 2: Access Windmill UI

Open http://localhost:8001

**Default credentials:**
- Email: `admin@windmill.dev`
- Password: `changeme`

### Step 3: Create a Workspace

1. Click "Create Workspace"
2. Name it: `demo`
3. Click "Create"

### Step 4: Create an API Token

1. Click your profile (top right)
2. Go to "Account Settings"
3. Click "Tokens" tab
4. Click "New Token"
5. Name: `api-token`
6. Expiration: Never (or your preference)
7. Click "Create"
8. **Copy the token** (you won't see it again!)

### Step 5: Deploy the Factorial Script

1. In Windmill UI, go to "Scripts" → "+ Script"
2. Choose "TypeScript (Deno)"
3. Set:
   - **Summary**: Distributed Factorial Calculator
   - **Path**: `u/admin/factorial`
4. Replace the template code with the content from `windmill-scripts/factorial.ts`
5. Click "Save"

### Step 6: Test the Script

1. In the script editor, click "Test"
2. Input:
   ```json
   {
     "number": 5,
     "request_id": "test-1"
   }
   ```
3. Click "Run"
4. You should see the result: `{"number": 5, "result": "120", ...}`

### Step 7: Enable Windmill Mode in API

Update `docker-compose.yml` for the `api` service:

```yaml
api:
  environment:
    USE_WINDMILL: "true"
    WINDMILL_TOKEN: "YOUR_TOKEN_FROM_STEP_4"
    # ... other env vars stay the same
```

### Step 8: Restart API Service

```bash
docker-compose restart api
```

Check logs:
```bash
docker-compose logs -f api
```

You should see:
```
[INFO] [api] Using Windmill orchestration windmill_url=http://windmill-server:8000 workspace=demo
```

### Step 9: Test via Frontend

1. Open http://localhost:3000
2. Enter a number (e.g., 10)
3. Click "Calculate Factorial"
4. Go to http://localhost:8001/runs
5. **You should see your job execution!**

## Viewing Runs in Windmill UI

After triggering calculations:

1. Go to http://localhost:8001/runs
2. You'll see all factorial calculation jobs
3. Click on a job to see:
   - Input parameters
   - Output result
   - Execution logs
   - Duration
   - Worker that processed it

## How Windmill Orchestration Works

When `USE_WINDMILL=true`:

1. **Frontend** → API `/calculate` endpoint
2. **API** → Triggers Windmill job `u/admin/factorial` via REST API
3. **Windmill** → Executes TypeScript script
4. **Script** → Makes NATS request to workers for calculation
5. **NATS Workers** → Process using distributed recursion
   - Each recursive step (n-1)! is a NATS request
   - Workers check NATS KV cache
   - Workers log to SurrealDB
6. **Script** → Returns result to Windmill
7. **Windmill** → Marks job as completed
8. **API** → Returns result to frontend

**Key benefit**: Every top-level factorial request becomes a trackable Windmill job with full observability!

## Troubleshooting

### API can't connect to Windmill

**Error**: `failed to run job: Windmill API returned 401`

**Solution**: Check `WINDMILL_TOKEN` is correct and not expired

### Script not found

**Error**: `failed to run job: Windmill API returned 404`

**Solution**:
1. Verify script exists at path `u/admin/factorial` in Windmill UI
2. Check `WINDMILL_SCRIPT_PATH` environment variable matches

### Jobs timeout

**Error**: `timeout waiting for job`

**Solution**:
1. Check NATS workers are running: `docker-compose ps factorial-worker`
2. Check NATS is accessible from Windmill: `docker-compose logs windmill-server`
3. Increase timeout in `windmill/client.go` if needed

## Switching Back to Direct NATS Mode

```yaml
api:
  environment:
    USE_WINDMILL: "false"
    # ... rest stays the same
```

Then restart:
```bash
docker-compose restart api
```

## Performance Comparison

**Direct NATS Mode:**
- Lower latency (~10-50ms for cached values)
- No job scheduling overhead
- No UI visibility

**Windmill Mode:**
- Higher latency (~100-500ms due to job scheduling)
- Full job tracking and observability
- View execution history
- Retry failed jobs
- Schedule periodic calculations

Choose based on your needs: **observability vs. performance**.
