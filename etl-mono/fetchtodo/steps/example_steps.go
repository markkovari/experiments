package steps

import (
	"encoding/json"
	"fmt"
	"log"
	"time"

	cloudevents "github.com/cloudevents/sdk-go/v2"
	ceevent "github.com/cloudevents/sdk-go/v2/event"
	"github.com/cucumber/godog"
	"github.com/nats-io/nats.go"
)

var (
	nc            *nats.Conn
	js            nats.JetStreamContext
	receivedEvent *ceevent.Event
)

func theNATSServerIsRunning() error {
	var err error
	nc, err = nats.Connect(nats.DefaultURL)
	if err != nil {
		return fmt.Errorf("could not connect to NATS: %w", err)
	}
	js, err = nc.JetStream()
	if err != nil {
		return fmt.Errorf("could not get JetStream context: %w", err)
	}

	_, _ = js.AddStream(&nats.StreamConfig{
		Name:     "CLOUD",
		Subjects: []string{"cloudevents.>"},
		Storage:  nats.MemoryStorage,
	})

	return nil
}

func iPublishACloudEventWithContent(subject string, content string) error {
	event := ceevent.New()
	event.SetSource("test")
	event.SetType("test.type")
	event.SetID("e2e-test-id")
	event.SetTime(time.Now())
	_ = event.SetData(cloudevents.ApplicationJSON, map[string]string{"message": content})

	data, err := json.Marshal(event)
	if err != nil {
		return err
	}

	_, err = js.Publish(subject, data)
	return err
}

func theMessageShouldBeReceivedByTheSubscriber() error {
	sub, err := js.PullSubscribe("cloudevents.>", "test-e2e")
	if err != nil {
		return fmt.Errorf("subscribe failed: %w", err)
	}

	msgs, err := sub.Fetch(1, nats.MaxWait(2*time.Second))
	if err != nil {
		return fmt.Errorf("message not received: %w", err)
	}

	for _, msg := range msgs {
		var event ceevent.Event
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			return fmt.Errorf("unmarshal failed: %w", err)
		}
		msg.Ack()
		receivedEvent = &event
		log.Printf("✅ Received event: %+v", event)
		return nil
	}
	return fmt.Errorf("no message received")
}

func InitializeScenario(ctx *godog.ScenarioContext) {
	ctx.Step(`^the NATS server is running$`, theNATSServerIsRunning)
	ctx.Step(`^I publish a "([^"]*)" event with content "([^"]*)"$`, iPublishACloudEventWithContent)
	ctx.Step(`^the message should be received by the subscriber$`, theMessageShouldBeReceivedByTheSubscriber)
}
