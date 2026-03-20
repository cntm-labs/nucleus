package admin

import (
	"fmt"
	"net/url"
	"strconv"
)

// Org represents a Nucleus organization returned by the Admin API.
type Org struct {
	ID        string                 `json:"id"`
	Name      string                 `json:"name"`
	Slug      string                 `json:"slug"`
	LogoURL   string                 `json:"logo_url,omitempty"`
	Metadata  map[string]interface{} `json:"metadata,omitempty"`
	CreatedAt string                 `json:"created_at"`
	UpdatedAt string                 `json:"updated_at"`
}

// ListOrgsParams controls pagination for listing organizations.
type ListOrgsParams struct {
	Limit  int
	Cursor string
}

// OrgsAPI provides access to the Nucleus Admin Orgs endpoints.
type OrgsAPI struct {
	client *HTTPClient
}

// NewOrgsAPI creates a new OrgsAPI instance.
func NewOrgsAPI(client *HTTPClient) *OrgsAPI {
	return &OrgsAPI{client: client}
}

// Get retrieves a single organization by ID.
func (a *OrgsAPI) Get(orgID string) (*Org, error) {
	var org Org
	if err := a.client.Get(fmt.Sprintf("/orgs/%s", orgID), &org); err != nil {
		return nil, err
	}
	return &org, nil
}

// List retrieves a paginated list of organizations.
func (a *OrgsAPI) List(params *ListOrgsParams) (*PaginatedResponse[Org], error) {
	query := url.Values{}
	if params != nil {
		if params.Limit > 0 {
			query.Set("limit", strconv.Itoa(params.Limit))
		}
		if params.Cursor != "" {
			query.Set("cursor", params.Cursor)
		}
	}

	path := "/orgs"
	if len(query) > 0 {
		path += "?" + query.Encode()
	}

	var resp PaginatedResponse[Org]
	if err := a.client.Get(path, &resp); err != nil {
		return nil, err
	}
	return &resp, nil
}
