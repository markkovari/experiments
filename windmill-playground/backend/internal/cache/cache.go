package cache

import (
	"context"
	"fmt"
	"math/big"
	"time"

	"github.com/nats-io/nats.go"
	"github.com/surrealdb/surrealdb.go"
)

type Cache struct {
	kv         nats.KeyValue    // NATS KV for caching with native TTL
	db         *surrealdb.DB    // SurrealDB for logging only
	defaultTTL time.Duration
}

type CalculationLog struct {
	ID           string    `json:"id,omitempty"`
	Number       int64     `json:"number"`
	Operation    string    `json:"operation"` // "factorial" or "product"
	Result       string    `json:"result"`
	WorkerID     string    `json:"worker_id"`
	CacheHit     bool      `json:"cache_hit"`
	Duration     int64     `json:"duration_ms"`
	CalculatedAt time.Time `json:"calculated_at"`
}

func New(natsURL, surrealURL, surrealUser, surrealPass, surrealNS, surrealDB string, ttl time.Duration) (*Cache, error) {
	// Connect to NATS for KV cache
	nc, err := nats.Connect(natsURL)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to NATS: %w", err)
	}

	// Create JetStream context
	js, err := nc.JetStream()
	if err != nil {
		return nil, fmt.Errorf("failed to create JetStream context: %w", err)
	}

	if ttl == 0 {
		ttl = 24 * time.Hour // Default to 24 hours
	}

	// Create or get KV bucket with TTL
	kv, err := js.CreateKeyValue(&nats.KeyValueConfig{
		Bucket:      "factorial_cache",
		Description: "Factorial calculation cache with TTL",
		TTL:         ttl, // Native TTL - entries auto-delete!
		Storage:     nats.MemoryStorage,
	})
	if err != nil {
		// Try to get existing bucket
		kv, err = js.KeyValue("factorial_cache")
		if err != nil {
			return nil, fmt.Errorf("failed to create/get KV bucket: %w", err)
		}
	}

	// Connect to SurrealDB for logging only
	db, err := surrealdb.New(surrealURL)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to SurrealDB: %w", err)
	}

	if _, err = db.Signin(map[string]interface{}{
		"user": surrealUser,
		"pass": surrealPass,
	}); err != nil {
		return nil, fmt.Errorf("failed to sign in to SurrealDB: %w", err)
	}

	if _, err = db.Use(surrealNS, surrealDB); err != nil {
		return nil, fmt.Errorf("failed to use namespace/database: %w", err)
	}

	return &Cache{
		kv:         kv,
		db:         db,
		defaultTTL: ttl,
	}, nil
}

func (c *Cache) GetFactorial(ctx context.Context, n int64) (*big.Int, bool, error) {
	key := fmt.Sprintf("factorial:%d", n)

	entry, err := c.kv.Get(key)
	if err != nil {
		if err == nats.ErrKeyNotFound {
			return nil, false, nil
		}
		return nil, false, fmt.Errorf("failed to get from cache: %w", err)
	}

	val := new(big.Int)
	if _, ok := val.SetString(string(entry.Value()), 10); !ok {
		return nil, false, fmt.Errorf("invalid cached result")
	}

	return val, true, nil
}

func (c *Cache) SetFactorial(ctx context.Context, n int64, result *big.Int) error {
	key := fmt.Sprintf("factorial:%d", n)
	value := []byte(result.String())

	_, err := c.kv.Put(key, value)
	if err != nil {
		return fmt.Errorf("failed to cache factorial: %w", err)
	}

	return nil
}

func (c *Cache) LogCalculation(ctx context.Context, log CalculationLog) error {
	log.CalculatedAt = time.Now()

	_, err := c.db.Create("calculation_log", log)
	if err != nil {
		return fmt.Errorf("failed to log calculation: %w", err)
	}

	return nil
}

func (c *Cache) Close() error {
	c.db.Close()
	return nil
}
