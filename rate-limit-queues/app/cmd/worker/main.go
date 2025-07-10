package main

import (
	"asdasd/events"
	"asdasd/models"
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"math/rand/v2"
	"os"
	"time"

	"github.com/rabbitmq/amqp091-go"
)

func handleMessage(msg amqp091.Delivery) (*events.Delayable, bool, error) {
	var user models.User
	err := json.Unmarshal(msg.Body, &user)
	if err != nil {
		return nil, false, err
	}

	slog.Info("event got", slog.Group("message details",
		slog.String("user name", string(user.Name)),
		slog.Int("passthroughId id", user.PassThroughID),
	))

	isRateLimited := rand.IntN(100)
	if isRateLimited < 10 {
		msg.Nack(false, true)
		afterTime := rand.IntN(10)
		slog.Info("delay",
			slog.Group("activated",
				slog.Int("with time", afterTime),
			))

		return &events.Delayable{
			RetryAfter: afterTime,
		}, false, nil
	}

	slog.Info("no delay",
		slog.Group("activated"))
	time.Sleep(12 * time.Second)

	if err := msg.Ack(false); err != nil {
		return nil, false, err
	}
	return nil, false, nil
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
