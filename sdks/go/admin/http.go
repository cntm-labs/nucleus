package admin

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

// HTTPClient handles authenticated requests to the Nucleus Admin API.
type HTTPClient struct {
	baseURL   string
	secretKey string
	client    *http.Client
}

// NewHTTPClient creates a new Admin API HTTP client.
func NewHTTPClient(baseURL, secretKey string) *HTTPClient {
	return &HTTPClient{
		baseURL:   baseURL,
		secretKey: secretKey,
		client:    &http.Client{},
	}
}

// APIError represents an error response from the Nucleus API.
type APIError struct {
	StatusCode int
	Code       string `json:"code"`
	Message    string `json:"message"`
}

func (e *APIError) Error() string {
	if e.Message != "" {
		return fmt.Sprintf("nucleus: API error %d: %s", e.StatusCode, e.Message)
	}
	return fmt.Sprintf("nucleus: API error %d", e.StatusCode)
}

func (c *HTTPClient) do(method, path string, result interface{}) error {
	url := c.baseURL + "/api/v1/admin" + path

	req, err := http.NewRequest(method, url, nil)
	if err != nil {
		return fmt.Errorf("nucleus: failed to create request: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+c.secretKey)

	resp, err := c.client.Do(req)
	if err != nil {
		return fmt.Errorf("nucleus: request failed: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("nucleus: failed to read response: %w", err)
	}

	if resp.StatusCode >= 400 {
		apiErr := &APIError{StatusCode: resp.StatusCode}
		if err := json.Unmarshal(body, apiErr); err == nil && apiErr.Message != "" {
			return apiErr
		}
		apiErr.Message = string(body)
		return apiErr
	}

	if result != nil {
		if err := json.Unmarshal(body, result); err != nil {
			return fmt.Errorf("nucleus: failed to decode response: %w", err)
		}
	}

	return nil
}

// Get performs a GET request to the given admin API path.
func (c *HTTPClient) Get(path string, result interface{}) error {
	return c.do(http.MethodGet, path, result)
}

// Post performs a POST request to the given admin API path.
func (c *HTTPClient) Post(path string, result interface{}) error {
	return c.do(http.MethodPost, path, result)
}

// Delete performs a DELETE request to the given admin API path.
func (c *HTTPClient) Delete(path string, result interface{}) error {
	return c.do(http.MethodDelete, path, result)
}
