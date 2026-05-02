package events

import (
	"context"
	"testing"
	"time"

	"github.com/nats-io/nats.go"
)

func TestNATSHandler_EndToEnd(t *testing.T) {
	// 1. Setup (Requires NATS running on :4222)
	handler, err := NewNATSHandler("nats://localhost:4222")
	if err != nil {
		t.Skip("NATS not available, skipping integration test")
	}
	defer handler.Close()

	streamName := "TEST_STREAM"
	subject := "test.event"
	_ = handler.CreateStream(streamName, []string{"test.*"})

	// 2. Subscribe
	received := make(chan []byte, 1)
	_, err = handler.SubscribeWithContext(subject, "test-group", func(ctx context.Context, msg *nats.Msg) {
		received <- msg.Data
		msg.Ack()
	})
	if err != nil {
		t.Fatalf("Failed to subscribe: %v", err)
	}

	// 3. Publish
	payload := []byte("hello-nats")
	err = handler.PublishWithContext(context.Background(), subject, payload)
	if err != nil {
		t.Fatalf("Failed to publish: %v", err)
	}

	// 4. Assert
	select {
	case data := <-received:
		if string(data) != "hello-nats" {
			t.Errorf("Expected 'hello-nats', got %s", string(data))
		}
	case <-time.After(2 * time.Second):
		t.Error("Timed out waiting for message")
	}
}
