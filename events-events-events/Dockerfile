FROM golang:1.25-alpine AS builder

WORKDIR /app

# Copy go mod and sum files
COPY go.mod go.sum ./
RUN go mod download

# Copy the source code
COPY . .

# Build Order Service
RUN go build -o /bin/order-service cmd/order-service/main.go

# Build Payment Service
RUN go build -o /bin/payment-service cmd/payment-service/main.go

# Build Shipping Service
RUN go build -o /bin/shipping-service cmd/shipping-service/main.go

# Build RPC Server
RUN go build -o /bin/rpc-server cmd/rpc-server/main.go

# Build Analytics Service
RUN go build -o /bin/analytics-service cmd/analytics-service/main.go

# Build Notification Service
RUN go build -o /bin/notification-service cmd/notification-service/main.go

# Final stage: Order Service
FROM alpine:latest AS order-service
COPY --from=builder /bin/order-service /app/order-service
CMD ["/app/order-service"]

# Final stage: Payment Service
FROM alpine:latest AS payment-service
COPY --from=builder /bin/payment-service /app/payment-service
CMD ["/app/payment-service"]

# Final stage: Shipping Service
FROM alpine:latest AS shipping-service
COPY --from=builder /bin/shipping-service /app/shipping-service
CMD ["/app/shipping-service"]

# Final stage: RPC Server
FROM alpine:latest AS rpc-server
COPY --from=builder /bin/rpc-server /app/rpc-server
EXPOSE 8080
CMD ["/app/rpc-server"]

# Final stage: Analytics Service
FROM alpine:latest AS analytics-service
COPY --from=builder /bin/analytics-service /app/analytics-service
CMD ["/app/analytics-service"]

# Final stage: Notification Service
FROM alpine:latest AS notification-service
COPY --from=builder /bin/notification-service /app/notification-service
CMD ["/app/notification-service"]
