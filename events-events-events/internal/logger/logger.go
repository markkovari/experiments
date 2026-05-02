package logger

import (
	"context"
	"log/slog"
	"os"

	"go.opentelemetry.io/otel/trace"
)

func New() *slog.Logger {
	return slog.New(slog.NewJSONHandler(os.Stdout, nil))
}

// WithTrace adds trace_id and span_id to the log attributes if they exist in the context
func WithTrace(ctx context.Context, l *slog.Logger) *slog.Logger {
	spanContext := trace.SpanContextFromContext(ctx)
	if spanContext.HasTraceID() {
		return l.With(
			slog.String("trace_id", spanContext.TraceID().String()),
			slog.String("span_id", spanContext.SpanID().String()),
		)
	}
	return l
}
