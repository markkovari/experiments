package events

import (
	"fmt"
	"log/slog"
	"os"
	"time"

	amqp "github.com/rabbitmq/amqp091-go"
)

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
