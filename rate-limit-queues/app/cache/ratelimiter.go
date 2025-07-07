package cache

import (
	"context"
	"fmt"
	"time"

	"github.com/valkey-io/valkey-go"
	"github.com/valkey-io/valkey-go/valkeylimiter"
)

type AppRateLimiter interface {
	AllowByUser(ctx context.Context, userID string) (bool, time.Duration, error)
	AllowByAPI(ctx context.Context, apiEndpoint string) (bool, time.Duration, error)
	Close() error
}

type valkeyAppRateLimiter struct {
	limiter   *valkeylimiter.RateLimiterClient
	client    valkey.Client
	userLimit uint
	apiLimit  uint
	duration  time.Duration
}

func NewValkeyAppRateLimiter(valkeyAddr string, options valkeylimiter.RateLimiterOption, userLimit, apiLimit uint) (AppRateLimiter, error) {
	valkeyClient, err := valkey.NewClient(valkey.ClientOption{InitAddress: []string{valkeyAddr}})
	if err != nil {
		return nil, fmt.Errorf("failed to connect to Valkey: %w", err)
	}

	limiter, err := valkeylimiter.NewRateLimiter(options)
	if err != nil {
		return nil, err
	}

	return &valkeyAppRateLimiter{
		limiter:   &limiter,
		client:    valkeyClient,
		userLimit: userLimit,
		apiLimit:  apiLimit,
	}, nil
}

func (v *valkeyAppRateLimiter) AllowByUser(ctx context.Context, userID string) (bool, time.Duration, error) {
	key := fmt.Sprintf("ratelimit:user:%s", userID)

	result, err := (*v.limiter).Allow(ctx, key, valkeylimiter.WithCustomRateLimit(int(v.userLimit), v.duration))
	if err != nil {
		return false, 0, fmt.Errorf("user rate limit check failed for %s: %w", userID, err)
	}
	return result.Allowed, time.Duration(result.ResetAtMs), nil
}

// AllowByAPI checks if an external API call is allowed based on the configured API rate limit.
func (v *valkeyAppRateLimiter) AllowByAPI(ctx context.Context, apiEndpoint string) (bool, time.Duration, error) {
	key := fmt.Sprintf("ratelimit:api:%s", apiEndpoint)
	result, err := (*v.limiter).Allow(ctx, key, valkeylimiter.WithCustomRateLimit(int(v.apiLimit), v.duration))
	if err != nil {
		return false, 0, fmt.Errorf("API rate limit check failed for %s: %w", apiEndpoint, err)
	}
	return result.Allowed, time.Duration(result.ResetAtMs), nil
}

func (v *valkeyAppRateLimiter) Close() error {
	v.client.Close()
	return nil
}
