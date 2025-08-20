package main

import (
	"fmt"
	"log"
	"time"

	"github.com/nats-io/nats.go"
)

func main() {
	nc, err := nats.Connect("nats://localhost:4222")
	if err != nil {
		log.Fatal(err)
	}
	defer nc.Close()

	_, err = nc.Subscribe("UserCreated", func(m *nats.Msg) {
		fmt.Printf("Subscriber 1 received a message: %s\n", string(m.Data))
	})
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Subscriber 1 is listening for UserCreated events...")
	select {} // Block forever
}
