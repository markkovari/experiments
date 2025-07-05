package main

import (
	"asdasd/events"
	"log/slog"

	"github.com/rabbitmq/amqp091-go"
)

func handleMessage(msg amqp091.Delivery) {
	slog.Info("event got", slog.Group("message details",
		slog.String("body_preview", string(msg.Body)),
	))

}

func main() {

	slog.Info("worker starts")

	if err := events.ConsumeMessagesFromChannel(handleMessage); err != nil {
		slog.Error("Cannot create message consumer")
	}

}
