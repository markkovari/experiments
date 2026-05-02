package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	"connectrpc.com/connect"
	orderv1 "github.com/markkovari/events-events-events/gen/proto/order/v1"
	"github.com/markkovari/events-events-events/gen/proto/order/v1/orderv1connect"
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
	// Rate limiting check
	if !s.limiter.Allow() {
		// Increment DROPPED counter
		s.reqCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("result", "dropped")))
		log.Printf("Rate limit exceeded: dropping request for customer %s", req.Msg.CustomerId)
		return nil, connect.NewError(connect.CodeResourceExhausted, fmt.Errorf("too many requests: limit is 15 msg/s"))
	}

	// Increment ALLOWED counter
	s.reqCounter.Add(ctx, 1, metric.WithAttributes(attribute.String("result", "allowed")))

	tracer := otel.Tracer("rpc-server")
	ctx, span := tracer.Start(ctx, "CreateOrder")
	defer span.End()

	orderID := fmt.Sprintf("order-%d", time.Now().UnixNano())
	log.Printf("Received CreateOrder request for customer: %s. Assigned ID: %s", req.Msg.CustomerId, orderID)

	event := OrderCreated{
		ID:         orderID,
		CustomerID: req.Msg.CustomerId,
		Amount:     req.Msg.Amount,
		CreatedAt:  time.Now().Format(time.RFC3339),
	}

	data, _ := json.Marshal(event)
	err := s.nats.PublishWithContext(ctx, "orders.created", data)
	if err != nil {
		log.Printf("Error publishing to NATS: %v", err)
	}

	res := connect.NewResponse(&orderv1.CreateOrderResponse{
		OrderId: orderID,
		Status:  "accepted",
	})
	return res, nil
}

func main() {
	cleanup, err := events.InitOTel("rpc-server")
	if err != nil {
		log.Printf("Failed to init OTel: %v", err)
	} else {
		defer cleanup()
	}

	natsURL := "nats://localhost:4222"
	if url := os.Getenv("NATS_URL"); url != "" {
		natsURL = url
	}

	natsHandler, err := events.NewNATSHandler(natsURL)
	if err != nil {
		log.Fatalf("Error connecting to NATS: %v", err)
	}
	defer natsHandler.Close()

	// Ensure stream exists
	_ = natsHandler.CreateStream("ORDERS", []string{"orders.*"})

	// Setup Meter and Counter
	meter := otel.Meter("rpc-server")
	counter, _ := meter.Int64Counter("order_requests_total",
		metric.WithDescription("Total number of order requests by result"),
	)

	limiter := rate.NewLimiter(rate.Every(time.Second/15), 1)

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

	log.Println("Starting ConnectRPC server on :8080 (Rate limit: 15 msg/s)")
	http.ListenAndServe(":8080", c.Handler(mux))
}
