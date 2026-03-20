package nucleus

import (
	"strings"
)

// GinContext is a minimal interface matching *gin.Context to avoid importing
// the Gin framework as a dependency. The real *gin.Context satisfies this.
type GinContext interface {
	GetHeader(key string) string
	Set(key string, value interface{})
	AbortWithStatusJSON(code int, jsonObj interface{})
	Next()
}

const ginClaimsKey = "nucleus_claims"

// GinMiddleware returns a Gin handler function that validates the Bearer token
// in the Authorization header. On success, it stores the parsed NucleusClaims
// in the Gin context under the key "nucleus_claims".
//
// Usage:
//
//	r.Use(nucleus.GinMiddleware(&nucleus.Config{SecretKey: "..."}))
func GinMiddleware(cfg *Config) func(GinContext) {
	return func(c GinContext) {
		auth := c.GetHeader("Authorization")
		if !strings.HasPrefix(auth, "Bearer ") {
			c.AbortWithStatusJSON(401, map[string]interface{}{
				"error": map[string]string{
					"code":    "auth/token_invalid",
					"message": "Missing authorization header",
				},
			})
			return
		}

		token := strings.TrimPrefix(auth, "Bearer ")
		claims, err := VerifyToken(token, cfg)
		if err != nil {
			c.AbortWithStatusJSON(401, map[string]interface{}{
				"error": map[string]string{
					"code":    "auth/token_invalid",
					"message": "Invalid or expired token",
				},
			})
			return
		}

		c.Set(ginClaimsKey, claims)
		c.Next()
	}
}

// ClaimsFromGin extracts the NucleusClaims stored by GinMiddleware.
// Returns nil if the request was not authenticated.
//
// Usage:
//
//	func handler(c *gin.Context) {
//	    claims := nucleus.ClaimsFromGin(c)
//	}
func ClaimsFromGin(c interface{ Get(string) (interface{}, bool) }) *NucleusClaims {
	val, exists := c.Get(ginClaimsKey)
	if !exists {
		return nil
	}
	claims, _ := val.(*NucleusClaims)
	return claims
}
