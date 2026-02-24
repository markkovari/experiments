#!/bin/bash
# Manual E2E Test Script for HTTP KV Counter
# Tests the component running on port 8081 (Docker) or 8080 (local)

PORT=${1:-8081}
BASE_URL="http://localhost:$PORT"

echo "🧪 Starting E2E Tests against $BASE_URL"
echo "=========================================="

# Test 1: POST /counter1 - Create and increment
echo ""
echo "Test 1: POST /counter1 (create)"
curl -s -X POST "$BASE_URL/counter1" | python3 -m json.tool

# Test 2: GET /counter1 - Get specific counter
echo ""
echo "Test 2: GET /counter1 (read)"
curl -s "$BASE_URL/counter1" | python3 -m json.tool

# Test 3: POST /counter1 again - Increment
echo ""
echo "Test 3: POST /counter1 (increment)"
curl -s -X POST "$BASE_URL/counter1" | python3 -m json.tool

# Test 4: POST /counter2 - Create another counter
echo ""
echo "Test 4: POST /counter2 (create another)"
curl -s -X POST "$BASE_URL/counter2" | python3 -m json.tool

# Test 5: GET / - Get all counters
echo ""
echo "Test 5: GET / (all counters)"
curl -s "$BASE_URL/" | python3 -m json.tool

# Test 6: TTL Test - Wait 4 seconds and check expiration
echo ""
echo "Test 6: Testing 3-second TTL (waiting 4 seconds...)"
sleep 4
echo "After 4 seconds - counters should be expired:"
curl -s "$BASE_URL/counter1" | python3 -m json.tool

echo ""
echo "=========================================="
echo "✅ Manual E2E Tests Complete!"
echo ""
echo "Expected Results:"
echo "- Tests 1-5: Should return valid counter JSON"
echo "- Test 6: Should return 404 error (counter expired)"
