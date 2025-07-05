package main

import (
	"asdasd/events"
	"context"
	"log/slog"
	"os"

	"github.com/rabbitmq/amqp091-go"
)

func handleMessage(msg amqp091.Delivery) {
	slog.Info("event got", slog.Group("message details",
		slog.String("body_preview", string(msg.Body)),
	))

}

func main() {

	slog.Info("worker starts")
	ctx := context.Background()
	if err := events.ConsumeMessagesFromChannelWithRateLimit(ctx, handleMessage); err != nil {
		slog.Error("Cannot read message with consumer")
		os.Exit(1)
	}

}
