package events

import (
	"context"
	"fmt"

	"github.com/nats-io/nats.go"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/propagation"
)

type NATSHandler struct {
	nc *nats.Conn
	js nats.JetStreamContext
}

func NewNATSHandler(url string) (*NATSHandler, error) {
	nc, err := nats.Connect(url)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to NATS: %w", err)
	}

	js, err := nc.JetStream()
	if err != nil {
		return nil, fmt.Errorf("failed to create JetStream context: %w", err)
	}

	return &NATSHandler{nc: nc, js: js}, nil
}

func (h *NATSHandler) CreateStream(streamName string, subjects []string) error {
	_, err := h.js.AddStream(&nats.StreamConfig{
		Name:     streamName,
		Subjects: subjects,
	})
	return err
}

func (h *NATSHandler) Publish(subject string, data []byte) error {
	_, err := h.js.Publish(subject, data)
	return err
}

func (h *NATSHandler) PublishWithContext(ctx context.Context, subject string, data []byte) error {
	msg := nats.NewMsg(subject)
	msg.Data = data
	otel.GetTextMapPropagator().Inject(ctx, propagation.HeaderCarrier(msg.Header))
	_, err := h.js.PublishMsg(msg)
	return err
}

func (h *NATSHandler) SubscribeWithContext(subject, queue string, handler func(ctx context.Context, msg *nats.Msg)) (*nats.Subscription, error) {
	return h.js.QueueSubscribe(subject, queue, func(msg *nats.Msg) {
		ctx := otel.GetTextMapPropagator().Extract(context.Background(), propagation.HeaderCarrier(msg.Header))
		handler(ctx, msg)
	})
}

// SubscribeWithThrottling creates a Durable consumer with a specific MaxAckPending limit
func (h *NATSHandler) SubscribeWithThrottling(subject, queue, durableName string, maxPending int, handler func(ctx context.Context, msg *nats.Msg)) (*nats.Subscription, error) {
	return h.js.QueueSubscribe(subject, queue, func(msg *nats.Msg) {
		ctx := otel.GetTextMapPropagator().Extract(context.Background(), propagation.HeaderCarrier(msg.Header))
		handler(ctx, msg)
	}, nats.Durable(durableName), nats.MaxAckPending(maxPending), nats.ManualAck())
}

func (h *NATSHandler) Close() {
	h.nc.Close()
}
