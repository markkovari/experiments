FROM golang:1.24-alpine AS builder

# Set the current working directory inside the container
WORKDIR /app

# Copy go mod and sum files to leverage Docker layer caching
COPY go.mod go.sum ./

# Download all dependencies. This step is cached if go.mod and go.sum don't change.
RUN go mod download

# Copy the rest of the application source code
COPY . .

# Build the Go application
# CGO_ENABLED=0 is crucial for static compilation, making the binary self-contained.
# GOOS=linux ensures the binary is built for Linux (the base image).
# -a -installsuffix cgo helps with static linking.
# -o main specifies the output executable name.
# -ldflags "-s -w" reduces the binary size by removing debug information.
RUN CGO_ENABLED=0 GOOS=linux go build ./cmd/server/main.go

# Use a minimal Alpine image for the final stage to create a small production image
FROM alpine:latest

# setup app user
RUN addgroup -g 1001 appuser && adduser -u 1001 -D -G appuser appuser

# Set the current working directory in the final image
WORKDIR /root/

# Copy the compiled application binary from the builder stage
COPY --from=builder --chown=appuser:appuser /app/main .

# make main runnable
RUN chmod +x main

# Set the user for subsequent commands and the CMD instruction
USER appuser
# Expose the port the application will listen on
EXPOSE 8080

# Command to run the application when the container starts
CMD ["./main"]
