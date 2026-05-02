package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/markkovari/events-events-events/internal/events"
	"github.com/nats-io/nats.go"
	"go.opentelemetry.io/otel"
)

type OrderCreated struct {
	ID         string  `json:"id"`
	CustomerID string  `json:"customer_id"`
	Amount     float64 `json:"amount"`
	CreatedAt  string  `json:"created_at"`
}

type PaymentProcessed struct {
	OrderID       string `json:"order_id"`
	TransactionID string `json:"transaction_id"`
	Status        string `json:"status"`
}

func main() {
	cleanup, err := events.InitOTel("payment-service")
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

	// Ensure stream exists
	err = handler.CreateStream("PAYMENTS", []string{"payments.*"})
	if err != nil {
		log.Printf("Stream might already exist: %v", err)
	}

	tracer := otel.Tracer("payment-service")

	_, err = handler.SubscribeWithContext("orders.created", "payment-service", func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "HandleOrderCreated")
		defer span.End()

		var event OrderCreated
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			log.Printf("Error unmarshaling event: %v", err)
			return
		}

		log.Printf("Processing payment for order: %s (Amount: %.2f)", event.ID, event.Amount)

		// Simulate payment processing
		paymentEvent := PaymentProcessed{
			OrderID:       event.ID,
			TransactionID: "tx-" + event.ID,
			Status:        "success",
		}

		data, _ := json.Marshal(paymentEvent)
		err = handler.PublishWithContext(ctx, "payments.processed", data)
		if err != nil {
			log.Printf("Error publishing payment event: %v", err)
			return
		}

		log.Printf("Payment processed for order: %s", event.ID)
		msg.Ack()
	})

	if err != nil {
		log.Fatalf("Error subscribing: %v", err)
	}

	log.Println("Payment Service started. Waiting for orders...")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
}
