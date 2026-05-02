package main

import (
	"context"
	"encoding/json"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/markkovari/events-events-events/internal/config"
	"github.com/markkovari/events-events-events/internal/events"
	"github.com/markkovari/events-events-events/internal/logger"
	"github.com/nats-io/nats.go"
	"go.opentelemetry.io/otel"
)

type OrderCreated struct {
	ID         string  `json:"id"`
	CustomerID string  `json:"customer_id"`
	Amount     float64 `json:"amount"`
}

type PaymentProcessed struct {
	OrderID       string `json:"order_id"`
	TransactionID string `json:"transaction_id"`
	Status        string `json:"status"`
}

func main() {
	cfg := config.Load()
	l := logger.New()

	cleanup, err := events.InitOTel("payment-service")
	if err == nil {
		defer cleanup()
	}

	handler, err := events.NewNATSHandler(cfg.NATSURL)
	if err != nil {
		l.Error("Failed to connect to NATS", "error", err)
		os.Exit(1)
	}

	tracer := otel.Tracer("payment-service")

	_, err = handler.SubscribeWithContext("orders.created", "payment-service", func(ctx context.Context, msg *nats.Msg) {
		ctx, span := tracer.Start(ctx, "HandleOrderCreated")
		defer span.End()

		// Get a logger with trace_id attached!
		tl := logger.WithTrace(ctx, l)

		var event OrderCreated
		if err := json.Unmarshal(msg.Data, &event); err != nil {
			tl.Error("Malformed event", "error", err)
			handler.MoveToDLQ(ctx, "orders.created", msg, err)
			return
		}

		tl.Info("Processing payment", "order_id", event.ID, "amount", event.Amount)

		paymentEvent := PaymentProcessed{
			OrderID:       event.ID,
			TransactionID: "tx-" + event.ID,
			Status:        "success",
		}

		data, _ := json.Marshal(paymentEvent)
		_ = handler.PublishWithContext(ctx, "payments.processed", data)

		tl.Info("Payment successful", "order_id", event.ID)
		msg.Ack()
	})

	l.Info("Payment Service ready")

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, syscall.SIGINT, syscall.SIGTERM)
	<-stop
}
