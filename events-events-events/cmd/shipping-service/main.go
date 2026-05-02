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

type PaymentProcessed struct {
	OrderID       string `json:"order_id"`
	TransactionID string `json:"transaction_id"`
	Status        string `json:"status"`
}

type ShippingUpdate struct {
	OrderID        string `json:"order_id"`
	TrackingNumber string `json:"tracking_number"`
	Status         string `json:"status"`
}

func main() {
	cleanup, err := events.InitOTel("shipping-service")
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
	err = handler.CreateStream("SHIPPING", []string{"shipping.*"})
	if err != nil {
		log.Printf("Stream might already exist: %v", err)
	}

	tracer := otel.Tracer("shipping-service")

	_, err = handler.SubscribeWithContext("payments.processed", "shipping-service", func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "HandlePaymentProcessed")
		defer span.End()

		var event PaymentProcessed
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			log.Printf("Error unmarshaling event: %v", err)
			return
		}

		if event.Status != "success" {
			log.Printf("Skipping shipping for failed payment: %s", event.OrderID)
			msg.Ack()
			return
		}

		log.Printf("Processing shipping for order: %s", event.OrderID)

		// Simulate shipping processing
		shippingEvent := ShippingUpdate{
			OrderID:        event.OrderID,
			TrackingNumber: "TRK-" + event.OrderID,
			Status:         "dispatched",
		}

		data, _ := json.Marshal(shippingEvent)
		err = handler.PublishWithContext(ctx, "shipping.status", data)
		if err != nil {
			log.Printf("Error publishing shipping event: %v", err)
			return
		}

		log.Printf("Shipping dispatched for order: %s", event.OrderID)
		msg.Ack()
	})

	if err != nil {
		log.Fatalf("Error subscribing: %v", err)
	}

	log.Println("Shipping Service started. Waiting for payments...")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
}
