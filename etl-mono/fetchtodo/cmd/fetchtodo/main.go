package main

import (
	"encoding/json"
	"log"
	"time"

	cloudevents "github.com/cloudevents/sdk-go/v2"
	ceevent "github.com/cloudevents/sdk-go/v2/event"
	"github.com/nats-io/nats.go"
)

func main() {
	nc, err := nats.Connect(nats.DefaultURL)
	if err != nil {
		log.Fatalf("connect error: %v", err)
	}
	defer nc.Drain()

	js, err := nc.JetStream()
	if err != nil {
		log.Fatalf("jetstream error: %v", err)
	}

	// Create stream (if not already exists)
	_, err = js.AddStream(&nats.StreamConfig{
		Name:     "CLOUD",
		Subjects: []string{"cloudevents.>"},
		Storage:  nats.FileStorage,
	})
	if err != nil && err != nats.ErrStreamNameAlreadyInUse {
		log.Fatalf("stream error: %v", err)
	}

	// Create a CloudEvent
	event := ceevent.New()
	event.SetSource("my-service")
	event.SetType("example.created")
	event.SetID("abc-123")
	event.SetTime(time.Now())
	_ = event.SetData(cloudevents.ApplicationJSON, map[string]string{
		"message": "Hello from CloudEvents over NATS!",
	})

	// Serialize CloudEvent to JSON
	data, err := json.Marshal(event)
	if err != nil {
		log.Fatalf("marshal error: %v", err)
	}

	// Publish to subject
	_, err = js.Publish("cloudevents.example.created", data)
	if err != nil {
		log.Fatalf("publish error: %v", err)
	}

	log.Println("CloudEvent published!")
}
