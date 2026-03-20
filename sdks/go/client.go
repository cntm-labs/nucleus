package nucleus

import (
	"github.com/nucleus-auth/nucleus-go/admin"
)

const defaultBaseURL = "https://api.nucleus.dev"

// Config holds the configuration for a NucleusClient.
type Config struct {
	// SecretKey is the admin/secret API key for your Nucleus project.
	SecretKey string

	// BaseURL overrides the default Nucleus API base URL.
	// If empty, defaults to "https://api.nucleus.dev".
	BaseURL string

	// JWKSCacheTTLSeconds controls how long the JWKS key set is cached.
	// If zero, defaults to 3600 (1 hour).
	JWKSCacheTTLSeconds int
}

func (c *Config) baseURL() string {
	if c.BaseURL != "" {
		return c.BaseURL
	}
	return defaultBaseURL
}

// NucleusClient is the main entry point for interacting with Nucleus.
type NucleusClient struct {
	config Config

	// Users provides access to the Admin Users API.
	Users *admin.UsersAPI

	// Orgs provides access to the Admin Orgs API.
	Orgs *admin.OrgsAPI
}

// NewClient creates a new NucleusClient with the given configuration.
func NewClient(cfg Config) *NucleusClient {
	baseURL := cfg.baseURL()
	httpClient := admin.NewHTTPClient(baseURL, cfg.SecretKey)

	return &NucleusClient{
		config: cfg,
		Users:  admin.NewUsersAPI(httpClient),
		Orgs:   admin.NewOrgsAPI(httpClient),
	}
}

// VerifyToken validates a JWT access token against the Nucleus JWKS endpoint
// and returns the parsed claims.
func (c *NucleusClient) VerifyToken(token string) (*NucleusClaims, error) {
	return VerifyToken(token, &c.config)
}
