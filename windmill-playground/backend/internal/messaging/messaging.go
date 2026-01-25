package messaging

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/nats-io/nats.go"
)

const (
	// Worker-facing NATS subjects
	FactorialRequestSubject  = "factorial.request"
	FactorialResponseSubject = "factorial.response"

	// API-facing NATS subjects
	APIRequestSubject  = "factorial.api.request"
	APIResponseSubject = "factorial.api.response"
)

type FactorialRequest struct {
	Number      int64  `json:"number"`
	RequestID   string `json:"request_id"`
	OriginalReq int64  `json:"original_request"` // Track the original request
}

type FactorialResponse struct {
	Number    int64  `json:"number"`
	RequestID string `json:"request_id"`
	Result    string `json:"result"`
	Error     string `json:"error,omitempty"`
}

// API-specific types
type CalculateRequest struct {
	Number int64 `json:"number"`
}

type CalculateResponse struct {
	RequestID string `json:"request_id"`
	Number    int64  `json:"number"`
	Result    string `json:"result,omitempty"`
	Error     string `json:"error,omitempty"`
	Status    string `json:"status"`
}

type NATSClient struct {
	conn *nats.Conn
}

func NewNATSClient(url string) (*NATSClient, error) {
	nc, err := nats.Connect(url)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to NATS: %w", err)
	}

	return &NATSClient{conn: nc}, nil
}

func (n *NATSClient) PublishRequest(ctx context.Context, req FactorialRequest) error {
	data, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("failed to marshal request: %w", err)
	}

	if err := n.conn.Publish(FactorialRequestSubject, data); err != nil {
		return fmt.Errorf("failed to publish request: %w", err)
	}

	return nil
}

func (n *NATSClient) PublishResponse(ctx context.Context, resp FactorialResponse) error {
	data, err := json.Marshal(resp)
	if err != nil {
		return fmt.Errorf("failed to marshal response: %w", err)
	}

	if err := n.conn.Publish(FactorialResponseSubject, data); err != nil {
		return fmt.Errorf("failed to publish response: %w", err)
	}

	return nil
}

func (n *NATSClient) SubscribeRequests(ctx context.Context, handler func(*FactorialRequest) error) error {
	_, err := n.conn.Subscribe(FactorialRequestSubject, func(msg *nats.Msg) {
		var req FactorialRequest
		if err := json.Unmarshal(msg.Data, &req); err != nil {
			log.Printf("Failed to unmarshal request: %v", err)
			return
		}

		if err := handler(&req); err != nil {
			log.Printf("Handler error: %v", err)
		}
	})

	if err != nil {
		return fmt.Errorf("failed to subscribe: %w", err)
	}

	return nil
}

func (n *NATSClient) SubscribeResponses(ctx context.Context, handler func(*FactorialResponse) error) error {
	_, err := n.conn.Subscribe(FactorialResponseSubject, func(msg *nats.Msg) {
		var resp FactorialResponse
		if err := json.Unmarshal(msg.Data, &resp); err != nil {
			log.Printf("Failed to unmarshal response: %v", err)
			return
		}

		if err := handler(&resp); err != nil {
			log.Printf("Handler error: %v", err)
		}
	})

	if err != nil {
		return fmt.Errorf("failed to subscribe: %w", err)
	}

	return nil
}

func (n *NATSClient) Close() {
	n.conn.Close()
}
