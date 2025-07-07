package main

import (
	"asdasd/events"
	"asdasd/models"
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/rabbitmq/amqp091-go"
)

func handleMessage(msg amqp091.Delivery) error {
	var user models.User
	err := json.Unmarshal(msg.Body, &user)
	if err != nil {
		return err
	}

	slog.Info("event got", slog.Group("message details",
		slog.String("user name", string(user.Name)),
		slog.Int("passthroughId id", user.PassThroughID),
	))
	time.Sleep(12 * time.Second)

	if err := msg.Ack(false); err != nil {
		return err
	}
	return nil
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
