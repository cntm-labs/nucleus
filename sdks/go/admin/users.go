package admin

import (
	"fmt"
	"net/url"
	"strconv"
)

// User represents a Nucleus user returned by the Admin API.
type User struct {
	ID            string                 `json:"id"`
	Email         string                 `json:"email"`
	EmailVerified bool                   `json:"email_verified"`
	Username      string                 `json:"username,omitempty"`
	FirstName     string                 `json:"first_name,omitempty"`
	LastName      string                 `json:"last_name,omitempty"`
	AvatarURL     string                 `json:"avatar_url,omitempty"`
	Metadata      map[string]interface{} `json:"metadata"`
	CreatedAt     string                 `json:"created_at"`
	UpdatedAt     string                 `json:"updated_at"`
}

// PaginatedResponse wraps a list endpoint response with cursor pagination.
type PaginatedResponse[T any] struct {
	Data       []T    `json:"data"`
	HasMore    bool   `json:"has_more"`
	NextCursor string `json:"next_cursor,omitempty"`
	TotalCount *int   `json:"total_count,omitempty"`
}

// ListUsersParams controls filtering and pagination for listing users.
type ListUsersParams struct {
	Limit         int
	Cursor        string
	EmailContains string
}

// UsersAPI provides access to the Nucleus Admin Users endpoints.
type UsersAPI struct {
	client *HTTPClient
}

// NewUsersAPI creates a new UsersAPI instance.
func NewUsersAPI(client *HTTPClient) *UsersAPI {
	return &UsersAPI{client: client}
}

// Get retrieves a single user by ID.
func (a *UsersAPI) Get(userID string) (*User, error) {
	var user User
	if err := a.client.Get(fmt.Sprintf("/users/%s", userID), &user); err != nil {
		return nil, err
	}
	return &user, nil
}

// List retrieves a paginated list of users.
func (a *UsersAPI) List(params *ListUsersParams) (*PaginatedResponse[User], error) {
	query := url.Values{}
	if params != nil {
		if params.Limit > 0 {
			query.Set("limit", strconv.Itoa(params.Limit))
		}
		if params.Cursor != "" {
			query.Set("cursor", params.Cursor)
		}
		if params.EmailContains != "" {
			query.Set("email_contains", params.EmailContains)
		}
	}

	path := "/users"
	if len(query) > 0 {
		path += "?" + query.Encode()
	}

	var resp PaginatedResponse[User]
	if err := a.client.Get(path, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}

// Delete removes a user by ID.
func (a *UsersAPI) Delete(userID string) error {
	return a.client.Delete(fmt.Sprintf("/users/%s", userID), nil)
}

// Ban disables a user account.
func (a *UsersAPI) Ban(userID string) error {
	return a.client.Post(fmt.Sprintf("/users/%s/ban", userID), nil)
}

// Unban re-enables a previously banned user account.
func (a *UsersAPI) Unban(userID string) error {
	return a.client.Post(fmt.Sprintf("/users/%s/unban", userID), nil)
}
