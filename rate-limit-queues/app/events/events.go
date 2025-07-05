package events

import (
	"asdasd/cache"
	"context"
	"fmt"
	"log"
	"log/slog"
	"os"
	"time"

	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/valkey-io/valkey-go/valkeylimiter"
)

type Message struct {
	UserID              string `json:"user_id"`
	ExternalAPIEndpoint string `json:"external_api_endpoint"`
	Payload             string `json:"payload"`
}

const QueueName = "some_thing_to_publish_to"

func CreateConn() (*amqp.Connection, error) {
	rabbitURL := os.Getenv("RABBITMQ_URL")
	if rabbitURL == "" {
		rabbitURL = "amqp://user:password@rabbitmq:5672/" // fallback default
	}
	fmt.Println("RabbitMQ URL:", rabbitURL)
	conn, err := amqp.Dial(rabbitURL)

	if err != nil {
		slog.Error("cannot connect to rabbitmq")
		return nil, err
	}

	return conn, nil
}

func InitQueue(queueName string) (*amqp.Queue, error) {
	conn, err := CreateConn()
	if err != nil {
		slog.Error("cannot open conn")
		return nil, err
	}

	ch, err := conn.Channel()
	if err != nil {
		slog.Error("cannot create channel")
		return nil, err
	}

	queue, err := ch.QueueDeclare(
		QueueName,
		true,
		false,
		false,
		false,
		nil,
	)
	if err != nil {
		return nil, err
	}

	return &queue, nil

}

func Send(message []byte) error {
	conn, err := CreateConn()
	if err != nil {
		slog.Error("cannot open conn")
		return err
	}

	ch, err := conn.Channel()
	if err != nil {
		slog.Error("cannot create channel")
		return err
	}

	defer ch.Close()
	defer conn.Close()
	err = SendMessageChannel(ch, message)
	if err != nil {
		return err
	}
	return nil
}

func SendMessageChannel(channel *amqp.Channel, message []byte) error {
	err := channel.Publish(
		"",
		QueueName,
		false,
		false,
		amqp.Publishing{
			ContentType: "text/plain",
			Body:        message,
			Timestamp:   time.Now(),
		})
	return err
}

type MessageHandler func(amqp.Delivery)

func ConsumeMessages(ch *amqp.Channel, handler MessageHandler) error {

	msgs, err := ch.Consume(
		QueueName,
		"",
		true,
		false,
		false,
		false,
		nil,
	)

	if err != nil {
		return err
	}
	forever := make(chan bool)
	go func() {
		for msg := range msgs {
			time.Sleep(5 * time.Second)
			handler(msg)
		}
	}()

	<-forever

	return err
}

func ConsumeMessagesFromChannel(handler MessageHandler) error {

	conn, err := CreateConn()
	if err != nil {
		slog.Error("cannot open conn")
		return err
	}

	ch, err := conn.Channel()
	if err != nil {
		slog.Error("cannot create channel")
		return err
	}

	defer ch.Close()
	defer conn.Close()
	msgs, err := ch.Consume(
		QueueName, // queue
		"",        // consumer
		true,      // auto-ack
		false,     // exclusive
		false,     // no-local
		false,     // no-wait
		nil,       // args
	)
	if err != nil {
		slog.Error("Cannot create messages from queue")
	}

	var forever chan struct{}

	go func() {
		for d := range msgs {
			handler(d)
		}
	}()

	slog.Info(" [*] Waiting for logs. To exit press CTRL+C")
	<-forever
	return nil
}

func ConsumeMessagesFromChannelWithRateLimit(ctx context.Context, handler MessageHandler) error {
	conn, err := CreateConn()
	if err != nil {
		slog.Error("cannot open conn")
		return err
	}

	ch, err := conn.Channel()
	if err != nil {
		slog.Error("cannot create channel")
		return err
	}

	for {
		msgs, err := ch.Consume(
			QueueName, // queue
			"",        // consumer
			true,      // auto-ack
			false,     // exclusive
			false,     // no-local
			false,     // no-wait
			nil,       // args
		)
		if err != nil {
			slog.Error("Cannot create messages from queue")
		}
		cacheAddress := os.Getenv("CACHE_ADDRESS")
		if cacheAddress == "" {
			cacheAddress = "localhost:6379"
		}
		appLimiter, err := cache.NewValkeyAppRateLimiter(cacheAddress, valkeylimiter.RateLimiterOption{}, 10, 15)
		if err != nil {
			slog.Error("cannot create rate limiter")
		}

		forever := make(chan bool)

		go func() {
			for d := range msgs {
				log.Printf("Received a message: %s", d.Body)

				msg := Message{
					UserID:              "user123",
					ExternalAPIEndpoint: "external-api-x",
					Payload:             string(d.Body),
				}

				go func(delivery amqp.Delivery) {
					ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
					defer cancel()

					allowedUser, userRetryAfter, err := appLimiter.AllowByUser(ctx, msg.UserID)
					if err != nil {
						slog.Info("Error checking user rate limit for %s: %v", msg.UserID, err)
						delivery.Nack(false, true)
						return
					}
					if !allowedUser {
						slog.Info("User %s rate limit exceeded. Retry after %s. Requeueing message.", msg.UserID, userRetryAfter)
						delivery.Nack(false, true)
						return
					}

					allowedAPI, apiRetryAfter, err := appLimiter.AllowByAPI(ctx, msg.ExternalAPIEndpoint)
					if err != nil {
						slog.Info("Error checking API rate limit for %s: %v", msg.ExternalAPIEndpoint, err)
						delivery.Nack(false, true)
						return
					}
					if !allowedAPI {
						slog.Info("External API %s rate limit exceeded. Retry after %s. Requeueing message.", msg.ExternalAPIEndpoint, apiRetryAfter)
						delivery.Nack(false, true)
						return
					}

					slog.Info(fmt.Sprintf("Processing message for user %s, API %s: %s", msg.UserID, msg.ExternalAPIEndpoint, msg.Payload))
					time.Sleep(500 * time.Millisecond)

					slog.Info("Message processed and acknowledged for user %s, API %s", msg.UserID, msg.ExternalAPIEndpoint)
					delivery.Ack(false)

				}(d)
			}
		}()

		<-forever
	}
}
