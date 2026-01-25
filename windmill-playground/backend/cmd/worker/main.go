package main

import (
	"context"
	"flag"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/google/uuid"
	"github.com/markkovari/windmill-playground/backend/internal/cache"
	"github.com/markkovari/windmill-playground/backend/internal/calculator"
	"github.com/markkovari/windmill-playground/backend/pkg/logger"
	"github.com/markkovari/windmill-playground/backend/internal/messaging"
)

func main() {
	// Command-line flags
	logLevel := flag.String("log-level", "", "Log level (debug, info, warn, error)")
	logJSON := flag.Bool("log-json", false, "Enable JSON logging")
	flag.Parse()

	// Get log configuration from flags or environment
	level := *logLevel
	if level == "" {
		level = logger.GetEnvWithDefault("LOG_LEVEL", logger.LevelInfo)
	}

	jsonLogging := *logJSON
	if !jsonLogging {
		jsonLogging = logger.ParseBool(os.Getenv("LOG_JSON"))
	}

	// Generate unique worker ID
	workerID := uuid.New().String()

	// Initialize logger
	log := logger.New("worker-"+workerID[:8], level, jsonLogging)
	log.Info("Starting factorial worker id=%s", workerID)

	// Get configuration from environment
	natsURL := logger.GetEnvWithDefault("NATS_URL", "nats://localhost:4222")
	surrealURL := logger.GetEnvWithDefault("SURREALDB_URL", "ws://localhost:8000/rpc")
	surrealUser := logger.GetEnvWithDefault("SURREALDB_USER", "root")
	surrealPass := logger.GetEnvWithDefault("SURREALDB_PASS", "root")
	surrealNS := logger.GetEnvWithDefault("SURREALDB_NS", "factorial")
	surrealDB := logger.GetEnvWithDefault("SURREALDB_DB", "calculations")
	cacheTTLStr := logger.GetEnvWithDefault("CACHE_TTL", "24h")

	// Parse cache TTL
	cacheTTL, err := time.ParseDuration(cacheTTLStr)
	if err != nil {
		log.Warn("Invalid CACHE_TTL '%s', using default 24h: %v", cacheTTLStr, err)
		cacheTTL = 24 * time.Hour
	}

	log.InfoWithFields("Configuration loaded", map[string]interface{}{
		"nats_url":    natsURL,
		"surreal_url": surrealURL,
		"cache_ttl":   cacheTTL.String(),
		"log_level":   level,
		"log_json":    jsonLogging,
	})

	// Initialize cache (NATS KV for caching, SurrealDB for logging)
	log.DebugWithFields("Initializing cache", map[string]interface{}{
		"nats_url":    natsURL,
		"surreal_url": surrealURL,
		"cache_ttl":   cacheTTL.String(),
	})
	cacheClient, err := cache.New(natsURL, surrealURL, surrealUser, surrealPass, surrealNS, surrealDB, cacheTTL)
	if err != nil {
		log.Fatal("Failed to create cache client: %v", err)
	}
	defer cacheClient.Close()

	// Initialize calculator
	calc := calculator.New(cacheClient, workerID, log)

	// Initialize NATS client
	log.Debug("Connecting to NATS at %s", natsURL)
	natsClient, err := messaging.NewNATSClient(natsURL)
	if err != nil {
		log.Fatal("Failed to create NATS client: %v", err)
	}
	defer natsClient.Close()

	// Subscribe to factorial requests
	ctx := context.Background()
	if err := natsClient.SubscribeRequests(ctx, func(req *messaging.FactorialRequest) error {
		log.InfoWithFields("Received factorial request", map[string]interface{}{
			"number":     req.Number,
			"request_id": req.RequestID,
		})

		result, err := calc.CalculateFactorial(ctx, req.Number)
		if err != nil {
			log.ErrorWithFields("Error calculating factorial", map[string]interface{}{
				"number":     req.Number,
				"request_id": req.RequestID,
				"error":      err.Error(),
			})

			// Publish error response
			natsClient.PublishResponse(ctx, messaging.FactorialResponse{
				Number:    req.Number,
				RequestID: req.RequestID,
				Error:     err.Error(),
			})
			return err
		}

		// Publish success response
		log.InfoWithFields("Calculated factorial", map[string]interface{}{
			"number":     req.Number,
			"result":     result.String(),
			"request_id": req.RequestID,
		})

		return natsClient.PublishResponse(ctx, messaging.FactorialResponse{
			Number:    req.Number,
			RequestID: req.RequestID,
			Result:    result.String(),
		})
	}); err != nil {
		log.Fatal("Failed to subscribe to requests: %v", err)
	}

	log.Info("Worker ready and listening for requests")

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	<-sigChan

	log.Info("Shutting down worker")
}
