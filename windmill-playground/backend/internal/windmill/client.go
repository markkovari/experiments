package windmill

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type Client struct {
	baseURL   string
	token     string
	workspace string
	httpClient *http.Client
}

type JobInput struct {
	Number    int64  `json:"number"`
	RequestID string `json:"request_id"`
}

type JobResult struct {
	Number   int64  `json:"number"`
	Result   string `json:"result"`
	CacheHit bool   `json:"cache_hit"`
	WorkerID string `json:"worker_id"`
	JobID    string `json:"job_id"`
	Error    string `json:"error,omitempty"`
}

type JobStatus struct {
	ID     string          `json:"id"`
	Type   string          `json:"type"`
	Result json.RawMessage `json:"result"`
	Logs   string          `json:"logs"`
}

func NewClient(baseURL, token, workspace string) *Client {
	return &Client{
		baseURL:   baseURL,
		token:     token,
		workspace: workspace,
		httpClient: &http.Client{
			Timeout: 60 * time.Second,
		},
	}
}

// RunJobAndWait triggers a Windmill job and waits for completion
func (c *Client) RunJobAndWait(ctx context.Context, scriptPath string, input JobInput) (*JobResult, error) {
	// Trigger the job
	jobID, err := c.runJob(ctx, scriptPath, input)
	if err != nil {
		return nil, fmt.Errorf("failed to run job: %w", err)
	}

	// Poll for completion
	return c.waitForJob(ctx, jobID)
}

func (c *Client) runJob(ctx context.Context, scriptPath string, input JobInput) (string, error) {
	url := fmt.Sprintf("%s/api/w/%s/jobs/run/p/%s", c.baseURL, c.workspace, scriptPath)

	body, err := json.Marshal(input)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(body))
	if err != nil {
		return "", err
	}

	req.Header.Set("Authorization", "Bearer "+c.token)
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("windmill API returned %d: %s", resp.StatusCode, string(bodyBytes))
	}

	var result struct {
		JobID string `json:"id"`
	}

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	// Windmill might return just the UUID string or a JSON object
	if err := json.Unmarshal(bodyBytes, &result); err != nil {
		// Try as plain string - trim quotes
		trimmed := bytes.Trim(bodyBytes, "\"")
		return string(trimmed), nil
	}

	return result.JobID, nil
}

func (c *Client) waitForJob(ctx context.Context, jobID string) (*JobResult, error) {
	url := fmt.Sprintf("%s/api/w/%s/jobs_u/completed/get/%s", c.baseURL, c.workspace, jobID)

	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	timeout := time.After(30 * time.Second)

	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timeout:
			return nil, fmt.Errorf("timeout waiting for job %s", jobID)
		case <-ticker.C:
			req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
			if err != nil {
				return nil, err
			}

			req.Header.Set("Authorization", "Bearer "+c.token)

			resp, err := c.httpClient.Do(req)
			if err != nil {
				continue // Keep polling
			}

			if resp.StatusCode == http.StatusOK {
				defer resp.Body.Close()
				var status JobStatus
				if err := json.NewDecoder(resp.Body).Decode(&status); err != nil {
					return nil, err
				}

				var result JobResult
				if err := json.Unmarshal(status.Result, &result); err != nil {
					return nil, fmt.Errorf("failed to unmarshal job result: %w", err)
				}

				result.JobID = jobID
				return &result, nil
			}

			resp.Body.Close()
			// Job not complete yet, continue polling
		}
	}
}
