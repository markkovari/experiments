package main

import (
	"context"
	"fmt"

	"time"

	"github.com/nats-io/nats.go"
	"github.com/nats-io/nats.go/jetstream"
)

func main() {

	nc, _ := nats.Connect(nats.DefaultURL)
	defer nc.Drain()

	js, _ := jetstream.New(nc)

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	bucketName := "user_notifications"
	notificationsBucket, err := js.KeyValue(ctx, bucketName)
	if err != nil {
		fmt.Println("Cannot get 'users_notifications'")
	}

	entry, err := notificationsBucket.Get(ctx, "markkovari")

	if err != nil {
		println("cannot get entry for markkovari")
	}
	println(string(entry.Value()))
}
