package config

import (
	"os"
	"time"
)

type Config struct {
	NATSURL       string
	OTelEndpoint  string
	Port          string
	LogLevel      string
	RateLimit     float64
	Burst         int
	ProcessDelay  time.Duration
}

func Load() *Config {
	return &Config{
		NATSURL:      getEnv("NATS_URL", "nats://localhost:4222"),
		OTelEndpoint: getEnv("OTEL_EXPORTER_OTLP_ENDPOINT", "localhost:4317"),
		Port:         getEnv("PORT", "8080"),
		LogLevel:     getEnv("LOG_LEVEL", "info"),
		// Default to 15 msg/s if not specified
		RateLimit:    20.0, 
		Burst:        100,
		// Simulation delay for analytics
		ProcessDelay: 200 * time.Millisecond,
	}
}

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
