package calculator

import (
	"context"
	"fmt"
	"math/big"
	"time"

	"github.com/google/uuid"
	"github.com/markkovari/windmill-playground/backend/internal/cache"
	"github.com/markkovari/windmill-playground/backend/internal/messaging"
	"github.com/markkovari/windmill-playground/backend/pkg/logger"
)

type Calculator struct {
	cache      *cache.Cache
	natsClient *messaging.NATSClient
	workerID   string
	logger     *logger.Logger
}

func New(c *cache.Cache, natsClient *messaging.NATSClient, workerID string, log *logger.Logger) *Calculator {
	return &Calculator{
		cache:      c,
		natsClient: natsClient,
		workerID:   workerID,
		logger:     log,
	}
}

// CalculateFactorial computes factorial recursively with caching
func (c *Calculator) CalculateFactorial(ctx context.Context, n int64) (*big.Int, error) {
	start := time.Now()

	// Check cache first
	if cached, found, err := c.cache.GetFactorial(ctx, n); err == nil && found {
		duration := time.Since(start).Milliseconds()

		c.logger.DebugWithFields("Cache hit for factorial", map[string]interface{}{
			"number":   n,
			"result":   cached.String(),
			"duration": duration,
		})

		// Log cache hit
		c.cache.LogCalculation(ctx, cache.CalculationLog{
			Number:    n,
			Operation: "factorial",
			Result:    cached.String(),
			WorkerID:  c.workerID,
			CacheHit:  true,
			Duration:  duration,
		})

		return cached, nil
	}

	c.logger.DebugWithFields("Cache miss for factorial", map[string]interface{}{
		"number": n,
	})

	// Base cases
	if n < 0 {
		return nil, fmt.Errorf("factorial not defined for negative numbers")
	}
	if n == 0 || n == 1 {
		result := big.NewInt(1)
		c.cache.SetFactorial(ctx, n, result)

		duration := time.Since(start).Milliseconds()
		c.cache.LogCalculation(ctx, cache.CalculationLog{
			Number:    n,
			Operation: "factorial",
			Result:    result.String(),
			WorkerID:  c.workerID,
			CacheHit:  false,
			Duration:  duration,
		})

		return result, nil
	}

	// Distributed recursive case: n! = n * (n-1)!
	// Instead of calling locally, make a distributed NATS request
	c.logger.DebugWithFields("Making distributed request for sub-factorial", map[string]interface{}{
		"parent_number":   n,
		"request_number":  n - 1,
		"requesting_from": "distributed_workers",
	})

	subRequestID := uuid.New().String()
	resp, err := c.natsClient.RequestFactorial(ctx, n-1, subRequestID)
	if err != nil {
		c.logger.ErrorWithFields("Failed to get distributed sub-factorial", map[string]interface{}{
			"parent_number":  n,
			"request_number": n - 1,
			"error":          err.Error(),
		})
		return nil, err
	}

	prevFactorial := new(big.Int)
	if _, ok := prevFactorial.SetString(resp.Result, 10); !ok {
		return nil, fmt.Errorf("invalid result from distributed calculation: %s", resp.Result)
	}

	c.logger.DebugWithFields("Received distributed sub-factorial result", map[string]interface{}{
		"parent_number":  n,
		"request_number": n - 1,
		"result":         resp.Result,
	})

	// Calculate product
	result := c.Product(ctx, n, prevFactorial)

	// Cache the result
	c.cache.SetFactorial(ctx, n, result)

	duration := time.Since(start).Milliseconds()

	c.logger.DebugWithFields("Calculated and cached factorial", map[string]interface{}{
		"number":   n,
		"result":   result.String(),
		"duration": duration,
	})

	c.cache.LogCalculation(ctx, cache.CalculationLog{
		Number:    n,
		Operation: "factorial",
		Result:    result.String(),
		WorkerID:  c.workerID,
		CacheHit:  false,
		Duration:  duration,
	})

	return result, nil
}

// Product computes n * prev and checks cache
func (c *Calculator) Product(ctx context.Context, n int64, prev *big.Int) *big.Int {
	start := time.Now()

	result := new(big.Int)
	result.Mul(big.NewInt(n), prev)

	duration := time.Since(start).Milliseconds()

	// Log product calculation
	c.cache.LogCalculation(ctx, cache.CalculationLog{
		Number:    n,
		Operation: "product",
		Result:    result.String(),
		WorkerID:  c.workerID,
		CacheHit:  false,
		Duration:  duration,
	})

	return result
}
