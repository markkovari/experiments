package events

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"time"

	"golang.org/x/time/rate"

	amqp "github.com/rabbitmq/amqp091-go"
	"github.com/valkey-io/valkey-go"
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
	cacheAddress := os.Getenv("CACHE_ADDRESS")
	if cacheAddress == "" {
		cacheAddress = "redis://user:password@localhost:6379/0?protocol=3"
	}

	limiterGlobal, err := valkeylimiter.NewRateLimiter(valkeylimiter.RateLimiterOption{
		ClientBuilder: func(option valkey.ClientOption) (valkey.Client, error) {
			return valkey.NewClient(valkey.MustParseURL(
				cacheAddress,
			))
		},
		KeyPrefix: "some-prefix",
		Limit:     15,
		Window:    10 * time.Second,
	})
	if err != nil {
		slog.Error("cannot create rate limiter")
		slog.Error(err.Error())
	}

	for {

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
				QueueName,
				"",
				false,
				false,
				false,
				false,
				nil,
			)
			if err != nil {
				slog.Error("Cannot create messages from queue")
			}
			slog.Info(" [*] Waiting for messages. To exit press CTRL+C")

			done := make(chan struct{})
			go func() {
				limiter := rate.NewLimiter(rate.Every(100*time.Millisecond), 1)

				for d := range msgs {
					if err := limiter.Wait(ctx); err == nil {
						msg := Message{
							UserID:              "user123",
							ExternalAPIEndpoint: "external-api-x",
							Payload:             string(d.Body),
						}

						go func(delivery amqp.Delivery) {
							time.Sleep(100 * time.Millisecond)

							ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
							defer cancel()

							result, err := limiterGlobal.Allow(ctx, msg.UserID)
							if err != nil {
								slog.Error("Rate limit check error", slog.String("err", err.Error()))
								delivery.Nack(false, true)
								return
							}

							if !result.Allowed {
								slog.Info(fmt.Sprintf("User %s rate limit exceeded. Retry after %d ms", msg.UserID, result.ResetAtMs))
								delivery.Nack(false, true)
								return
							}

							slog.Info(fmt.Sprintf("Processing message for user %s", msg.UserID))
							handler(d)
						}(d)
					}
				}
				close(done)
			}()

			<-done
			slog.Warn("⚠️ Message consumer closed, reconnecting...")

			ch.Close()
			conn.Close()
			time.Sleep(1 * time.Second)
		}
	}
}
