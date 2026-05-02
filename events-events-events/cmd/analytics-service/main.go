package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/markkovari/events-events-events/internal/config"
	"github.com/markkovari/events-events-events/internal/events"
	"github.com/nats-io/nats.go"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

type OrderCreated struct {
	ID         string  `json:"id"`
	CustomerID string  `json:"customer_id"`
	Amount     float64 `json:"amount"`
	CreatedAt  string  `json:"created_at"`
}

func main() {
	cfg := config.Load()

	// 1. Init Observability
	cleanup, err := events.InitOTel("analytics-service")
	if err != nil {
		log.Printf("Failed to init OTel: %v", err)
	}

	// 2. Connect to NATS
	handler, err := events.NewNATSHandler(cfg.NATSURL)
	if err != nil {
		log.Fatalf("Error connecting to NATS: %v", err)
	}

	meter := otel.Meter("analytics-service")
	counter, _ := meter.Int64Counter("analytics_processed_total")
	tracer := otel.Tracer("analytics-service")

	// 3. Subscribe with backpressure
	sub, err := handler.SubscribeWithThrottling("orders.created", "analytics-service", "analytics-backpressure", 1, func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "ProcessAnalytics")
		defer span.End()

		var event OrderCreated
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			return
		}

		log.Printf("Processing: %s", event.ID)
		time.Sleep(cfg.ProcessDelay)

		counter.Add(ctx, 1, metric.WithAttributes(attribute.String("status", "success")))
		msg.Ack()
	})

	if err != nil {
		log.Fatalf("Error subscribing: %v", err)
	}

	log.Println("Analytics Service running. Press Ctrl+C to stop gracefully.")

	// 4. GRACEFUL SHUTDOWN LOGIC
	stop := make(chan os.Signal, 1)
	signal.Notify(stop, syscall.SIGINT, syscall.SIGTERM)

	<-stop // Wait for signal

	log.Println("Shutting down gracefully...")

	// Stop receiving new messages
	sub.Unsubscribe()

	// Give NATS time to process remaining acks and OTel to flush
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	handler.Close()
	if cleanup != nil {
		cleanup()
	}

	log.Println("Service stopped.")
}
