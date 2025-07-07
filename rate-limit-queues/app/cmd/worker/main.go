package main

import (
	"asdasd/events"
	"context"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/rabbitmq/amqp091-go"
)

func handleMessage(msg amqp091.Delivery) {
	slog.Info("event got", slog.Group("message details",
		slog.String("body_preview", string(msg.Body)),
	))
	time.Sleep(2 * time.Second)

	msg.Ack(false)
}

func main() {

	slog.Info("worker starts")
	slog.Info("creating the queue if does not exists")
	if q, err := events.InitQueue(events.QueueName); err != nil {
		slog.Error("Cannot (re)create channel")
		os.Exit(1)
	} else {
		slog.Info(fmt.Sprintf("queue re-created %s", q.Name))
	}
	ctx := context.Background()
	if err := events.ConsumeMessagesFromChannelWithRateLimit(ctx, handleMessage); err != nil {
		slog.Error("Cannot read message with consumer")
		os.Exit(1)
	}

}
