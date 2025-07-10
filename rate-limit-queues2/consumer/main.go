package main

import (
	"context"
	"fmt"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

func main() {
	ctx := context.Background()
	nc, err := nats.Connect("nats://localhost:4222")
	if err != nil {
		panic("Cannot connect to nats")
	}
	js, err := jetstream.New(nc)
	if err != nil {
		panic("Cannot connect to js")

	}
	cons2, err := js.CreateOrUpdateConsumer(ctx, "TASKS", jetstream.ConsumerConfig{
		Name:          "task_high_consumer",
		Durable:       "task_high_consumer",
		FilterSubject: "tasks.high",
		AckPolicy:     jetstream.AckExplicitPolicy,
	})

	if err != nil {
		println(err.Error())
		fmt.Println("Cannot create or update consumer")
	}
	_, err = cons2.Consume(func(msg jetstream.Msg) {
		println(string(msg.Data()))
		msg.Ack()
	})

}
