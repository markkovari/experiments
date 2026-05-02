package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"time"

	"connectrpc.com/connect"
	orderv1 "github.com/markkovari/events-events-events/gen/proto/order/v1"
	"github.com/markkovari/events-events-events/gen/proto/order/v1/orderv1connect"
	"github.com/markkovari/events-events-events/internal/config"
	"github.com/markkovari/events-events-events/internal/events"
	"github.com/rs/cors"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
	"golang.org/x/time/rate"
)

type OrderCreated struct {
	ID         string  `json:"id"`
	CustomerID string  `json:"customer_id"`
	Amount     float64 `json:"amount"`
	CreatedAt  string  `json:"created_at"`
}

type OrderServer struct {
	nats       *events.NATSHandler
	limiter    *rate.Limiter
	reqCounter metric.Int64Counter
}

func (s *OrderServer) CreateOrder(
	ctx context.Context,
	req *connect.Request[orderv1.CreateOrderRequest],
) (*connect.Response[orderv1.CreateOrderResponse], error) {
	if !s.limiter.Allow() {
		s.reqCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("result", "dropped")))
		return nil, connect.NewError(connect.CodeResourceExhausted, fmt.Errorf("too many requests"))
	}

	s.reqCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("result", "allowed")))

	tracer := otel.Tracer("rpc-server")
	ctx, span := tracer.Start(ctx, "CreateOrder")
	defer span.End()

	orderID := fmt.Sprintf("order-%d", time.Now().UnixNano())
	event := OrderCreated{
		ID:         orderID,
		CustomerID: req.Msg.CustomerId,
		Amount:     req.Msg.Amount,
		CreatedAt:  time.Now().Format(time.RFC3339),
	}

	data, _ := json.Marshal(event)
	_ = s.nats.PublishWithContext(ctx, "orders.created", data)

	return connect.NewResponse(&orderv1.CreateOrderResponse{
		OrderId: orderID,
		Status:  "accepted",
	}), nil
}

func main() {
	cfg := config.Load()

	cleanup, err := events.InitOTel("rpc-server")
	if err == nil {
		defer cleanup()
	}

	natsHandler, err := events.NewNATSHandler(cfg.NATSURL)
	if err != nil {
		log.Fatalf("Error connecting to NATS: %v", err)
	}
	defer natsHandler.Close()

	_ = natsHandler.CreateStream("ORDERS", []string{"orders.*"})

	meter := otel.Meter("rpc-server")
	counter, _ := meter.Int64Counter("order_requests_total")

	limiter := rate.NewLimiter(rate.Limit(cfg.RateLimit), cfg.Burst)

	server := &OrderServer{
		nats:       natsHandler,
		limiter:    limiter,
		reqCounter: counter,
	}
	mux := http.NewServeMux()
	path, handler := orderv1connect.NewOrderServiceHandler(server)
	mux.Handle(path, handler)

	c := cors.New(cors.Options{
		AllowedOrigins: []string{"*"},
		AllowedMethods: []string{"GET", "POST", "OPTIONS"},
		AllowedHeaders: []string{"Connect-Protocol-Version", "Content-Type"},
	})

	log.Printf("Starting RPC server on :%s", cfg.Port)
	http.ListenAndServe(":"+cfg.Port, c.Handler(mux))
}
