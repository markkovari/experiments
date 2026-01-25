package logger

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strings"
	"time"
)

// Log levels
const (
	LevelDebug = "debug"
	LevelInfo  = "info"
	LevelWarn  = "warn"
	LevelError = "error"
)

var levelPriority = map[string]int{
	LevelDebug: 0,
	LevelInfo:  1,
	LevelWarn:  2,
	LevelError: 3,
}

type Logger struct {
	level      string
	component  string
	enableJSON bool
	logger     *log.Logger
}

type logEntry struct {
	Level     string    `json:"level"`
	Component string    `json:"component"`
	Message   string    `json:"msg"`
	Timestamp time.Time `json:"timestamp"`
	Fields    map[string]interface{} `json:"fields,omitempty"`
}

func New(component, level string, enableJSON bool) *Logger {
	if level == "" {
		level = LevelInfo
	}
	level = strings.ToLower(level)

	if _, ok := levelPriority[level]; !ok {
		level = LevelInfo
	}

	return &Logger{
		level:      level,
		component:  component,
		enableJSON: enableJSON,
		logger:     log.New(os.Stdout, "", 0),
	}
}

func (l *Logger) shouldLog(level string) bool {
	return levelPriority[level] >= levelPriority[l.level]
}

func (l *Logger) log(level, format string, args ...interface{}) {
	if !l.shouldLog(level) {
		return
	}

	msg := fmt.Sprintf(format, args...)

	if l.enableJSON {
		entry := logEntry{
			Level:     level,
			Component: l.component,
			Message:   msg,
			Timestamp: time.Now().UTC(),
		}
		data, _ := json.Marshal(entry)
		l.logger.Println(string(data))
	} else {
		levelUpper := strings.ToUpper(level)
		l.logger.Printf("[%s] [%s] %s", levelUpper, l.component, msg)
	}
}

func (l *Logger) logWithFields(level, msg string, fields map[string]interface{}) {
	if !l.shouldLog(level) {
		return
	}

	if l.enableJSON {
		entry := logEntry{
			Level:     level,
			Component: l.component,
			Message:   msg,
			Timestamp: time.Now().UTC(),
			Fields:    fields,
		}
		data, _ := json.Marshal(entry)
		l.logger.Println(string(data))
	} else {
		levelUpper := strings.ToUpper(level)
		fieldsStr := ""
		for k, v := range fields {
			fieldsStr += fmt.Sprintf(" %s=%v", k, v)
		}
		l.logger.Printf("[%s] [%s] %s%s", levelUpper, l.component, msg, fieldsStr)
	}
}

func (l *Logger) Debug(format string, args ...interface{}) {
	l.log(LevelDebug, format, args...)
}

func (l *Logger) DebugWithFields(msg string, fields map[string]interface{}) {
	l.logWithFields(LevelDebug, msg, fields)
}

func (l *Logger) Info(format string, args ...interface{}) {
	l.log(LevelInfo, format, args...)
}

func (l *Logger) InfoWithFields(msg string, fields map[string]interface{}) {
	l.logWithFields(LevelInfo, msg, fields)
}

func (l *Logger) Warn(format string, args ...interface{}) {
	l.log(LevelWarn, format, args...)
}

func (l *Logger) WarnWithFields(msg string, fields map[string]interface{}) {
	l.logWithFields(LevelWarn, msg, fields)
}

func (l *Logger) Error(format string, args ...interface{}) {
	l.log(LevelError, format, args...)
}

func (l *Logger) ErrorWithFields(msg string, fields map[string]interface{}) {
	l.logWithFields(LevelError, msg, fields)
}

func (l *Logger) Fatal(format string, args ...interface{}) {
	l.log(LevelError, format, args...)
	os.Exit(1)
}

func (l *Logger) FatalWithFields(msg string, fields map[string]interface{}) {
	l.logWithFields(LevelError, msg, fields)
	os.Exit(1)
}

// GetEnvWithDefault gets environment variable with fallback
func GetEnvWithDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

// ParseBool parses boolean from string
func ParseBool(s string) bool {
	s = strings.ToLower(s)
	return s == "true" || s == "1" || s == "yes"
}
