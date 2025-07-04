package events

import (
	"log/slog"
	"time"

	amqp "github.com/rabbitmq/amqp091-go"
)

const QueueName = "some_thing_to_publish_to"

func CreateConn() (*amqp.Connection, error) {
	conn, err := amqp.Dial("amqp://user:password@localhost:5672/")

	if err != nil {
		slog.Error("cannot connect to rabbitmq")
		return nil, err
	}

	defer conn.Close()
	return conn, nil
}

func CreateQueue(queueName string) (*amqp.Queue, error) {
	conn, err := CreateConn()
	if err != nil {

		slog.Error("cannot open conn")
		return nil, err
	}

	ch, err := conn.Channel()
	if err != nil {
		slog.Error("cannot crete channel")
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

func SendMessageChannel(channel amqp.Channel, message []byte) error {
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
