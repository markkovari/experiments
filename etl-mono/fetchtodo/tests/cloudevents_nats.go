package natscloudevents_nats

import (
	"encoding/json"
	"fmt"
	"time"

	ceevent "github.com/cloudevents/sdk-go/v2/event"
	"github.com/nats-io/nats.go"
)

type CloudEventNATS struct {
	js nats.JetStreamContext
	nc *nats.Conn
}

func NewCloudEventNATS(url string) (*CloudEventNATS, error) {
	nc, err := nats.Connect(url)
	if err != nil {
		return nil, err
	}
	js, err := nc.JetStream()
	if err != nil {
		return nil, err
	}
	return &CloudEventNATS{js: js, nc: nc}, nil
}

func (c *CloudEventNATS) Close() {
	_ = c.nc.Drain()
}

func (c *CloudEventNATS) EnsureStream(name string, subjects []string) error {
	_, err := c.js.AddStream(&nats.StreamConfig{
		Name:     name,
		Subjects: subjects,
		Storage:  nats.FileStorage,
	})
	if err != nil && err != nats.ErrStreamNameAlreadyInUse {
		return err
	}
	return nil
}

func (c *CloudEventNATS) Publish(subject string, event ceevent.Event) error {
	data, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("marshal CloudEvent: %w", err)
	}
	_, err = c.js.Publish(subject, data)
	return err
}

func (c *CloudEventNATS) Subscribe(
	subject string,
	consumer string,
	handler func(ceevent.Event),
) error {
	sub, err := c.js.PullSubscribe(subject, consumer)
	if err != nil {
		return err
	}

	go func() {
		for {
			msgs, err := sub.Fetch(10, nats.MaxWait(2*time.Second))
			if err != nil {
				continue // retry loop
			}
			for _, msg := range msgs {
				var evt ceevent.Event
				if err := json.Unmarshal(msg.Data, &evt); err != nil {
					_ = msg.Nak()
					continue
				}
				handler(evt)
				_ = msg.Ack()
			}
		}
	}()

	return nil
}
