package main

import (
	"encoding/json"
	"flag"
	"net/http"
	"os"
	"time"

	"github.com/google/uuid"
	"github.com/markkovari/windmill-playground/backend/internal/messaging"
	"github.com/markkovari/windmill-playground/backend/pkg/logger"
	"github.com/nats-io/nats.go"
)

var (
	nc  *nats.Conn
	log *logger.Logger
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
	log = logger.New("api", level, jsonLogging)

	// Connect to NATS
	natsURL := logger.GetEnvWithDefault("NATS_URL", "nats://localhost:4222")
	var err error
	nc, err = nats.Connect(natsURL)
	if err != nil {
		log.Fatal("Failed to connect to NATS: %v", err)
	}
	defer nc.Close()

	log.InfoWithFields("Connected to NATS", map[string]interface{}{
		"url": natsURL,
	})

	// Setup HTTP server
	http.HandleFunc("/calculate", enableCORS(calculateHandler))
	http.HandleFunc("/health", enableCORS(healthHandler))

	port := logger.GetEnvWithDefault("PORT", "8080")
	log.InfoWithFields("API server starting", map[string]interface{}{
		"port":      port,
		"log_level": level,
		"log_json":  jsonLogging,
	})

	if err := http.ListenAndServe(":"+port, nil); err != nil {
		log.Fatal("Server failed: %v", err)
	}
}

func enableCORS(next http.HandlerFunc) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type")

		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}

		next(w, r)
	}
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
	json.NewEncoder(w).Encode(map[string]string{"status": "ok"})
}

func calculateHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req messaging.CalculateRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		log.Warn("Invalid request body: %v", err)
		http.Error(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	if req.Number < 0 {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(messaging.CalculateResponse{
			Status: "error",
			Error:  "Number must be non-negative",
		})
		return
	}

	// Generate request ID
	requestID := uuid.New().String()

	log.InfoWithFields("Incoming calculation request", map[string]interface{}{
		"number":     req.Number,
		"request_id": requestID,
	})

	// Subscribe to response from Windmill orchestrator
	responseChan := make(chan messaging.FactorialResponse, 1)
	sub, err := nc.Subscribe(messaging.APIResponseSubject, func(msg *nats.Msg) {
		var resp messaging.FactorialResponse
		if err := json.Unmarshal(msg.Data, &resp); err == nil {
			if resp.RequestID == requestID {
				responseChan <- resp
			}
		}
	})
	if err != nil {
		log.Error("Failed to subscribe to responses: %v", err)
		http.Error(w, "Internal server error", http.StatusInternalServerError)
		return
	}
	defer sub.Unsubscribe()

	// Publish request to Windmill orchestrator
	factorialReq := messaging.FactorialRequest{
		Number:      req.Number,
		RequestID:   requestID,
		OriginalReq: req.Number,
	}

	reqData, err := json.Marshal(factorialReq)
	if err != nil {
		log.Error("Failed to marshal request: %v", err)
		http.Error(w, "Failed to marshal request", http.StatusInternalServerError)
		return
	}

	if err := nc.Publish(messaging.APIRequestSubject, reqData); err != nil {
		log.Error("Failed to publish request: %v", err)
		http.Error(w, "Failed to publish request", http.StatusInternalServerError)
		return
	}

	log.DebugWithFields("Published request to orchestrator", map[string]interface{}{
		"number":     req.Number,
		"request_id": requestID,
	})

	// Wait for response with timeout
	select {
	case resp := <-responseChan:
		w.Header().Set("Content-Type", "application/json")
		if resp.Error != "" {
			log.WarnWithFields("Calculation error", map[string]interface{}{
				"request_id": requestID,
				"error":      resp.Error,
			})
			json.NewEncoder(w).Encode(messaging.CalculateResponse{
				RequestID: requestID,
				Number:    req.Number,
				Status:    "error",
				Error:     resp.Error,
			})
		} else {
			log.InfoWithFields("Calculation successful", map[string]interface{}{
				"request_id": requestID,
				"number":     req.Number,
				"result":     resp.Result,
			})
			json.NewEncoder(w).Encode(messaging.CalculateResponse{
				RequestID: requestID,
				Number:    req.Number,
				Result:    resp.Result,
				Status:    "success",
			})
		}
	case <-time.After(30 * time.Second):
		log.ErrorWithFields("Timeout waiting for response", map[string]interface{}{
			"request_id": requestID,
			"number":     req.Number,
		})
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(messaging.CalculateResponse{
			RequestID: requestID,
			Number:    req.Number,
			Status:    "error",
			Error:     "Timeout waiting for response",
		})
	}
}
