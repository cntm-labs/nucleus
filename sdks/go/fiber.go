package nucleus

import (
	"strings"
)

// FiberContext is a minimal interface matching *fiber.Ctx to avoid importing
// the Fiber framework as a dependency. The real *fiber.Ctx satisfies this.
type FiberContext interface {
	Get(key string, defaultValue ...string) string
	Locals(key interface{}, value ...interface{}) interface{}
	Status(status int) FiberContext
	JSON(data interface{}) error
	Next() error
}

const fiberClaimsKey = "nucleus_claims"

// FiberMiddleware returns a Fiber handler that validates the Bearer token in the
// Authorization header. On success, it stores the parsed NucleusClaims in
// Fiber locals under the key "nucleus_claims".
//
// Usage:
//
//	app.Use(nucleus.FiberMiddleware(&nucleus.Config{SecretKey: "..."}))
func FiberMiddleware(cfg *Config) func(FiberContext) error {
	return func(c FiberContext) error {
		auth := c.Get("Authorization")
		if !strings.HasPrefix(auth, "Bearer ") {
			return c.Status(401).JSON(map[string]interface{}{
				"error": map[string]string{
					"code":    "auth/token_invalid",
					"message": "Missing authorization header",
				},
			})
		}

		token := strings.TrimPrefix(auth, "Bearer ")
		claims, err := VerifyToken(token, cfg)
		if err != nil {
			return c.Status(401).JSON(map[string]interface{}{
				"error": map[string]string{
					"code":    "auth/token_invalid",
					"message": "Invalid or expired token",
				},
			})
		}

		c.Locals(fiberClaimsKey, claims)
		return c.Next()
	}
}

// ClaimsFromFiber extracts the NucleusClaims stored by FiberMiddleware.
// Returns nil if the request was not authenticated.
//
// Usage:
//
//	func handler(c *fiber.Ctx) error {
//	    claims := nucleus.ClaimsFromFiber(c)
//	}
func ClaimsFromFiber(c interface{ Locals(interface{}, ...interface{}) interface{} }) *NucleusClaims {
	val := c.Locals(fiberClaimsKey)
	if val == nil {
		return nil
	}
	claims, _ := val.(*NucleusClaims)
	return claims
}
