package nucleus

import "github.com/golang-jwt/jwt/v5"

// NucleusClaims represents the JWT claims issued by Nucleus.
type NucleusClaims struct {
	jwt.RegisteredClaims

	Email          string                 `json:"email,omitempty"`
	FirstName      string                 `json:"first_name,omitempty"`
	LastName       string                 `json:"last_name,omitempty"`
	AvatarURL      string                 `json:"avatar_url,omitempty"`
	EmailVerified  bool                   `json:"email_verified,omitempty"`
	Metadata       map[string]interface{} `json:"metadata,omitempty"`
	OrgID          string                 `json:"org_id,omitempty"`
	OrgSlug        string                 `json:"org_slug,omitempty"`
	OrgRole        string                 `json:"org_role,omitempty"`
	OrgPermissions []string               `json:"org_permissions,omitempty"`
}

// UserID returns the subject claim, which is the Nucleus user ID.
func (c *NucleusClaims) UserID() string {
	return c.Subject
}
