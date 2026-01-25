package calculator

import (
	"context"
	"fmt"
	"math/big"
	"time"

	"github.com/markkovari/windmill-playground/backend/internal/cache"
	"github.com/markkovari/windmill-playground/backend/pkg/logger"
)

type Calculator struct {
	cache    *cache.Cache
	workerID string
	logger   *logger.Logger
}

func New(c *cache.Cache, workerID string, log *logger.Logger) *Calculator {
	return &Calculator{
		cache:    c,
		workerID: workerID,
		logger:   log,
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

	// Recursive case: n! = n * (n-1)!
	prevFactorial, err := c.CalculateFactorial(ctx, n-1)
	if err != nil {
		return nil, err
	}

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
