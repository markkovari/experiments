package inner

import (
	"context"
	"encoding/json"
	"fmt"
	"math/big"
	"net/http"
	"os"
	"time"

	"github.com/nats-io/nats.go"
)

// Input for the factorial script
type Input struct {
	Number    int64  `json:"number"`
	RequestID string `json:"request_id"`
}

// Output from the factorial script
type Output struct {
	Number    int64  `json:"number"`
	Result    string `json:"result"`
	CacheHit  bool   `json:"cache_hit"`
	WorkerID  string `json:"worker_id"`
	Error     string `json:"error,omitempty"`
}

// CalculationLog for SurrealDB
type CalculationLog struct {
	Number       int64     `json:"number"`
	Operation    string    `json:"operation"`
	Result       string    `json:"result"`
	WorkerID     string    `json:"worker_id"`
	CacheHit     bool      `json:"cache_hit"`
	Duration     int64     `json:"duration_ms"`
	CalculatedAt time.Time `json:"calculated_at"`
}

// Main is the entrypoint for the Windmill script
func Main(input Input) (Output, error) {
	start := time.Now()
	workerID := os.Getenv("WM_JOB_ID") // Windmill provides job ID
	if workerID == "" {
		workerID = "windmill-unknown"
	}

	ctx := context.Background()

	// Connect to NATS for KV cache
	natsURL := getEnv("NATS_URL", "nats://nats:4222")
	nc, err := nats.Connect(natsURL)
	if err != nil {
		return Output{Error: fmt.Sprintf("failed to connect to NATS: %v", err)}, err
	}
	defer nc.Close()

	js, err := nc.JetStream()
	if err != nil {
		return Output{Error: fmt.Sprintf("failed to get JetStream: %v", err)}, err
	}

	// Get or create KV bucket
	kv, err := js.KeyValue("factorial_cache")
	if err != nil {
		return Output{Error: fmt.Sprintf("failed to get KV bucket: %v", err)}, err
	}

	// Check cache first
	cacheKey := fmt.Sprintf("factorial:%d", input.Number)
	entry, err := kv.Get(cacheKey)
	if err == nil {
		// Cache hit!
		duration := time.Since(start).Milliseconds()
		logCalculation(ctx, CalculationLog{
			Number:       input.Number,
			Operation:    "factorial",
			Result:       string(entry.Value()),
			WorkerID:     workerID,
			CacheHit:     true,
			Duration:     duration,
			CalculatedAt: time.Now(),
		})

		return Output{
			Number:   input.Number,
			Result:   string(entry.Value()),
			CacheHit: true,
			WorkerID: workerID,
		}, nil
	}

	// Cache miss - calculate
	if input.Number < 0 {
		return Output{Error: "factorial not defined for negative numbers"},
			fmt.Errorf("factorial not defined for negative numbers")
	}

	if input.Number == 0 || input.Number == 1 {
		result := "1"
		kv.Put(cacheKey, []byte(result))

		duration := time.Since(start).Milliseconds()
		logCalculation(ctx, CalculationLog{
			Number:       input.Number,
			Operation:    "factorial",
			Result:       result,
			WorkerID:     workerID,
			CacheHit:     false,
			Duration:     duration,
			CalculatedAt: time.Now(),
		})

		return Output{
			Number:   input.Number,
			Result:   result,
			CacheHit: false,
			WorkerID: workerID,
		}, nil
	}

	// Recursive case: call Windmill for (n-1)!
	prevResult, err := callWindmillFactorial(input.Number - 1)
	if err != nil {
		return Output{Error: fmt.Sprintf("failed to calculate sub-factorial: %v", err)}, err
	}

	// Calculate n * (n-1)!
	prev := new(big.Int)
	prev.SetString(prevResult, 10)

	n := big.NewInt(input.Number)
	result := new(big.Int).Mul(n, prev)
	resultStr := result.String()

	// Cache the result
	kv.Put(cacheKey, []byte(resultStr))

	duration := time.Since(start).Milliseconds()
	logCalculation(ctx, CalculationLog{
		Number:       input.Number,
		Operation:    "factorial",
		Result:       resultStr,
		WorkerID:     workerID,
		CacheHit:     false,
		Duration:     duration,
		CalculatedAt: time.Now(),
	})

	return Output{
		Number:   input.Number,
		Result:   resultStr,
		CacheHit: false,
		WorkerID: workerID,
	}, nil
}

// callWindmillFactorial makes a recursive call to Windmill
func callWindmillFactorial(n int64) (string, error) {
	// Use Windmill API to trigger the same script
	windmillURL := getEnv("WINDMILL_URL", "http://windmill-server:8000")
	token := os.Getenv("WM_TOKEN") // Windmill provides this in scripts
	workspace := getEnv("WM_WORKSPACE", "demo")

	// Call the script via Windmill API
	url := fmt.Sprintf("%s/api/w/%s/jobs/run/p/u/admin/factorial", windmillURL, workspace)

	input := Input{Number: n, RequestID: fmt.Sprintf("sub-%d", n)}
	body, _ := json.Marshal(input)

	req, err := http.NewRequest("POST", url, nil)
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", "Bearer "+token)
	req.Header.Set("Content-Type", "application/json")

	// For now, use NATS request instead (simpler than full Windmill recursion)
	// In production, you'd use Windmill's job API with proper waiting
	return callViaNATS(n)
}

// callViaNATS is a fallback to use NATS workers directly
func callViaNATS(n int64) (string, error) {
	natsURL := getEnv("NATS_URL", "nats://nats:4222")
	nc, err := nats.Connect(natsURL)
	if err != nil {
		return "", err
	}
	defer nc.Close()

	type Request struct {
		Number      int64  `json:"number"`
		RequestID   string `json:"request_id"`
		OriginalReq int64  `json:"original_request"`
	}

	type Response struct {
		Result string `json:"result"`
		Error  string `json:"error,omitempty"`
	}

	req := Request{
		Number:      n,
		RequestID:   fmt.Sprintf("windmill-sub-%d", n),
		OriginalReq: n,
	}

	data, _ := json.Marshal(req)
	msg, err := nc.Request("factorial.request", data, 30*time.Second)
	if err != nil {
		return "", err
	}

	var resp Response
	if err := json.Unmarshal(msg.Data, &resp); err != nil {
		return "", err
	}

	if resp.Error != "" {
		return "", fmt.Errorf(resp.Error)
	}

	return resp.Result, nil
}

func logCalculation(ctx context.Context, log CalculationLog) {
	// Log to SurrealDB
	// For Windmill scripts, we could use HTTP calls to SurrealDB REST API
	// Or keep using the workers to log via NATS
	// Simplified for now
}

func getEnv(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}
