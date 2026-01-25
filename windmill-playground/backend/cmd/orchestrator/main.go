package main

import (
	"context"
	"encoding/json"
	"flag"
	"os"

	"github.com/markkovari/windmill-playground/backend/internal/messaging"
	"github.com/markkovari/windmill-playground/backend/pkg/logger"
	"github.com/nats-io/nats.go"
)

func main() {
	// Command-line flags
	logLevel := flag.String("log-level", "", "Log level (debug, info, warn, error)")
	logJSON := flag.Bool("log-json", false, "Enable JSON logging")
	flag.Parse()

	// Get log configuration
	level := *logLevel
	if level == "" {
		level = logger.GetEnvWithDefault("LOG_LEVEL", logger.LevelInfo)
	}

	jsonLogging := *logJSON
	if !jsonLogging {
		jsonLogging = logger.ParseBool(os.Getenv("LOG_JSON"))
	}

	// Initialize logger
	log := logger.New("orchestrator", level, jsonLogging)

	natsURL := os.Getenv("NATS_URL")
	if natsURL == "" {
		natsURL = "nats://nats:4222"
	}

	// Connect to NATS
	nc, err := nats.Connect(natsURL)
	if err != nil {
		log.Fatal("Failed to connect to NATS: %v", err)
	}
	defer nc.Close()

	log.InfoWithFields("Windmill orchestrator started", map[string]interface{}{
		"nats_url":  natsURL,
		"log_level": level,
		"log_json":  jsonLogging,
	})

	ctx := context.Background()

	// Subscribe to incoming requests from API
	_, err = nc.Subscribe(messaging.APIRequestSubject, func(msg *nats.Msg) {
		var req messaging.FactorialRequest
		if err := json.Unmarshal(msg.Data, &req); err != nil {
			log.Error("Failed to unmarshal API request: %v", err)
			return
		}

		log.InfoWithFields("Received request from API", map[string]interface{}{
			"number":     req.Number,
			"request_id": req.RequestID,
		})

		// Forward to workers using shared messaging client
		reqData, err := json.Marshal(req)
		if err != nil {
			log.Error("Failed to marshal worker request: %v", err)
			return
		}

		// Publish to workers using the constant from messaging package
		if err := nc.Publish(messaging.FactorialRequestSubject, reqData); err != nil {
			log.Error("Failed to publish to workers: %v", err)
			return
		}

		log.DebugWithFields("Forwarded request to workers", map[string]interface{}{
			"request_id": req.RequestID,
		})
	})

	if err != nil {
		log.Fatal("Failed to subscribe to API requests: %v", err)
	}

	// Subscribe to worker responses and forward back to API
	_, err = nc.Subscribe(messaging.FactorialResponseSubject, func(msg *nats.Msg) {
		var resp messaging.FactorialResponse
		if err := json.Unmarshal(msg.Data, &resp); err != nil {
			log.Error("Failed to unmarshal worker response: %v", err)
			return
		}

		log.DebugWithFields("Received response from worker", map[string]interface{}{
			"request_id": resp.RequestID,
			"result":     resp.Result,
		})

		// Forward to API response channel
		respData, err := json.Marshal(resp)
		if err != nil {
			log.Error("Failed to marshal API response: %v", err)
			return
		}

		if err := nc.Publish(messaging.APIResponseSubject, respData); err != nil {
			log.Error("Failed to publish API response: %v", err)
			return
		}

		log.InfoWithFields("Forwarded response to API", map[string]interface{}{
			"request_id": resp.RequestID,
			"result":     resp.Result,
		})
	})

	if err != nil {
		log.Fatal("Failed to subscribe to worker responses: %v", err)
	}

	log.Info("Orchestrator ready and listening...")

	// Keep running
	_ = ctx // Use context to avoid unused variable warning
	select {}
}
