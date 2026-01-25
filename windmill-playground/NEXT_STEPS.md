# ✅ Configuration Updated - Next Steps

## What Was Done

### 1. Services Started ✅
All services are running:
- Windmill UI: http://localhost:8001
- API: http://localhost:8080
- Frontend: http://localhost:3000
- NATS: running with 3 workers

### 2. Environment Variables Updated ✅

**docker-compose.yml changes:**
- ✅ Added `NATS_URL: nats://nats:4222` to windmill-server
- ✅ Added `NATS_URL: nats://nats:4222` to windmill-worker
- ✅ API configured with Windmill environment variables:
  - `USE_WINDMILL: "false"` (ready to enable)
  - `WINDMILL_URL: http://windmill-server:8000`
  - `WINDMILL_TOKEN: ""` (needs your token)
  - `WINDMILL_WORKSPACE: demo`
  - `WINDMILL_SCRIPT_PATH: u/admin/factorial`

### 3. Windmill services restarted ✅
Both windmill-server and windmill-worker have been restarted with the new NATS configuration.

---

## What You Need to Do

### Option A: Use the Automated Script (Recommended)

Run the enable script which will guide you through all steps:

```bash
./ENABLE_WINDMILL.sh
```

This script will:
1. Check if services are running
2. Guide you to create a Windmill token
3. Prompt you to deploy the factorial script
4. Update docker-compose.yml with your token
5. Restart the API service
6. Show you the results

### Option B: Manual Setup

Follow these steps:

#### Step 1: Complete Windmill Setup

1. Open http://localhost:8001
2. Complete initial setup (create admin account)
3. Create workspace named `demo`

#### Step 2: Create API Token

1. Click your profile → Account Settings
2. Go to Tokens tab
3. Click "New Token"
   - Name: `api-token`
   - Expiration: Never
4. **Copy the token**

#### Step 3: Deploy Factorial Script

1. Go to Scripts → + Script
2. Choose "TypeScript (Deno)"
3. Path: `u/admin/factorial`
4. Copy content from `windmill-scripts/factorial.ts`
5. Click Save
6. Test with: `{"number": 5, "request_id": "test-1"}`

#### Step 4: Update Configuration

Edit `docker-compose.yml` and find the `api` service:

```yaml
api:
  environment:
    USE_WINDMILL: "true"  # Change from false to true
    WINDMILL_TOKEN: "YOUR_TOKEN_HERE"  # Paste your token
```

#### Step 5: Restart API

```bash
docker-compose restart api
```

#### Step 6: Verify

```bash
docker-compose logs api | grep -i windmill
```

You should see:
```
[INFO] [api] Using Windmill orchestration windmill_url=http://windmill-server:8000 workspace=demo
```

---

## Testing Windmill Mode

### 1. Via Frontend

1. Open http://localhost:3000
2. Enter a number (e.g., 10)
3. Click "Calculate Factorial"
4. **Go to http://localhost:8001/runs**
5. You should see your job!

### 2. Via API

```bash
curl -X POST http://localhost:8080/calculate \
  -H "Content-Type: application/json" \
  -d '{"number": 5}'
```

Then check runs: http://localhost:8001/runs

---

## What You'll See in Windmill

Once enabled, every factorial calculation will appear as a job in the Windmill UI with:

- ✅ Input parameters (number, request_id)
- ✅ Output result (factorial value)
- ✅ Execution logs
- ✅ Duration
- ✅ Worker that processed it
- ✅ Full execution timeline

---

## Troubleshooting

### Services not running?
```bash
docker-compose ps
docker-compose up -d
```

### API not connecting to Windmill?
Check token is correct:
```bash
docker-compose logs api | grep -i "windmill\|token"
```

### Script not found?
Verify script exists at `u/admin/factorial` in Windmill UI

### Still using NATS mode?
Check environment variable:
```bash
docker-compose exec api env | grep USE_WINDMILL
```

Should show: `USE_WINDMILL=true`

---

## Documentation

- **Full setup guide**: `WINDMILL_SETUP.md`
- **Quick start**: `README.md`
- **This file**: `NEXT_STEPS.md`

---

## Current Status

🟡 **Windmill Mode: READY TO ENABLE**

Everything is configured and ready. You just need to:
1. Get a Windmill token from the UI
2. Deploy the TypeScript script
3. Update docker-compose.yml with the token
4. Restart the API

Run `./ENABLE_WINDMILL.sh` to do this interactively!
