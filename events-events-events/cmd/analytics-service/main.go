package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

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
	cleanup, err := events.InitOTel("analytics-service")
	if err != nil {
		log.Printf("Failed to init OTel: %v", err)
	} else {
		defer cleanup()
	}

	natsURL := "nats://localhost:4222"
	if url := os.Getenv("NATS_URL"); url != "" {
		natsURL = url
	}

	handler, err := events.NewNATSHandler(natsURL)
	if err != nil {
		log.Fatalf("Error connecting to NATS: %v", err)
	}
	defer handler.Close()

	// Setup Meter and Counter
	meter := otel.Meter("analytics-service")
	counter, _ := meter.Int64Counter("analytics_processed_total",
		metric.WithDescription("Total number of analytics events processed"),
	)

	tracer := otel.Tracer("analytics-service")

	// Throttling at the NATS layer:
	// MaxAckPending: 1 means NATS will not send message #2 until message #1 is Acknowledged.
	// This creates "Natural Backpressure".
	_, err = handler.SubscribeWithThrottling("orders.created", "analytics-service", "analytics-backpressure", 1, func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "ProcessAnalytics")
		defer span.End()

		var event OrderCreated
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			log.Printf("Error unmarshaling event: %v", err)
			return
		}

		// Simulate "Work" that takes time. 
		// Because MaxAckPending is 1, NATS will wait for this handler to finish and Ack 
		// before sending the next message.
		log.Printf("Analytics processing (Heavy Work) for order: %s", event.ID)
		time.Sleep(200 * time.Millisecond) // This results in ~5 msgs per second

		counter.Add(ctx, 1, metric.WithAttributes(attribute.String("status", "success")))
		
		msg.Ack()
	})

	if err != nil {
		log.Fatalf("Error subscribing: %v", err)
	}

	log.Println("Analytics Service started. Throttling via NATS MaxAckPending...")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
}
