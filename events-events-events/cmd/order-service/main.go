package main

import (
	"encoding/json"
	"log"
	"os"
	"time"

	"github.com/markkovari/events-events-events/internal/events"
)

type OrderCreated struct {
	ID         string  `json:"id"`
	CustomerID string  `json:"customer_id"`
	Amount     float64 `json:"amount"`
	CreatedAt  string  `json:"created_at"`
}

func main() {
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
	err = handler.CreateStream("ORDERS", []string{"orders.*"})
	if err != nil {
		log.Printf("Stream might already exist: %v", err)
	}

	event := OrderCreated{
		ID:         "order-123",
		CustomerID: "cust-456",
		Amount:     99.99,
		CreatedAt:  time.Now().Format(time.RFC3339),
	}

	data, _ := json.Marshal(event)
	err = handler.Publish("orders.created", data)
	if err != nil {
		log.Fatalf("Error publishing event: %v", err)
	}

	log.Println("Published order.created event successfully")
}
