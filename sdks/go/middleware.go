package nucleus

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
)

type contextKey struct{}

// Protect returns a net/http middleware that validates the Bearer token in the
// Authorization header. On success, it stores the parsed NucleusClaims in the
// request context. On failure, it responds with 401 Unauthorized.
//
// Use ClaimsFromContext to retrieve the claims in downstream handlers.
func Protect(cfg *Config) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			auth := r.Header.Get("Authorization")
			if !strings.HasPrefix(auth, "Bearer ") {
				writeUnauthorized(w, "Missing authorization header")
				return
			}

			token := strings.TrimPrefix(auth, "Bearer ")
			claims, err := VerifyToken(token, cfg)
			if err != nil {
				writeUnauthorized(w, "Invalid or expired token")
				return
			}

			ctx := context.WithValue(r.Context(), contextKey{}, claims)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// ClaimsFromContext extracts the NucleusClaims from a request context.
// Returns nil if the request was not authenticated.
func ClaimsFromContext(ctx context.Context) *NucleusClaims {
	claims, _ := ctx.Value(contextKey{}).(*NucleusClaims)
	return claims
}

func writeUnauthorized(w http.ResponseWriter, message string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusUnauthorized)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"code":    "auth/token_invalid",
			"message": message,
		},
	})
}
