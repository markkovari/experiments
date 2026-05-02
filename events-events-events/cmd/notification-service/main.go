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

type ShippingUpdate struct {
	OrderID        string `json:"order_id"`
	TrackingNumber string `json:"tracking_number"`
	Status         string `json:"status"`
}

func main() {
	cleanup, err := events.InitOTel("notification-service")
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

	tracer := otel.Tracer("notification-service")

	_, err = handler.SubscribeWithContext("shipping.status", "notification-service", func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "SendNotification")
		defer span.End()

		var event ShippingUpdate
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			log.Printf("Error unmarshaling event: %v", err)
			return
		}

		log.Printf("NOTIFIED: Order %s is now %s (Tracking: %s)", event.OrderID, event.Status, event.TrackingNumber)
		msg.Ack()
	})

	if err != nil {
		log.Fatalf("Error subscribing: %v", err)
	}

	log.Println("Notification Service started. Consuming shipping updates...")

	sig := make(chan os.Signal, 1)
	signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
	<-sig
}
