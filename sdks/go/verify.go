package nucleus

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/MicahParks/keyfunc/v3"
	"github.com/golang-jwt/jwt/v5"
)

var (
	jwksMu    sync.RWMutex
	jwksCache keyfunc.Keyfunc
	jwksURL   string
	jwksExp   time.Time
)

// VerifyToken validates a JWT access token against the Nucleus JWKS endpoint
// and returns the parsed claims. It caches the JWKS key set for the duration
// specified by Config.JWKSCacheTTLSeconds (default 1 hour).
func VerifyToken(token string, cfg *Config) (*NucleusClaims, error) {
	kf, err := getKeyfunc(cfg)
	if err != nil {
		return nil, fmt.Errorf("nucleus: failed to fetch JWKS: %w", err)
	}

	parsed, err := jwt.ParseWithClaims(token, &NucleusClaims{}, kf.KeyfuncLegacy, jwt.WithValidMethods([]string{"RS256"}))
	if err != nil {
		return nil, fmt.Errorf("nucleus: invalid token: %w", err)
	}

	claims, ok := parsed.Claims.(*NucleusClaims)
	if !ok {
		return nil, fmt.Errorf("nucleus: unexpected claims type")
	}

	return claims, nil
}

func getKeyfunc(cfg *Config) (keyfunc.Keyfunc, error) {
	url := cfg.baseURL() + "/.well-known/jwks.json"
	ttl := time.Duration(cfg.JWKSCacheTTLSeconds) * time.Second
	if ttl == 0 {
		ttl = time.Hour
	}

	jwksMu.RLock()
	if jwksCache != nil && jwksURL == url && time.Now().Before(jwksExp) {
		kf := jwksCache
		jwksMu.RUnlock()
		return kf, nil
	}
	jwksMu.RUnlock()

	jwksMu.Lock()
	defer jwksMu.Unlock()

	// Double-check after acquiring write lock.
	if jwksCache != nil && jwksURL == url && time.Now().Before(jwksExp) {
		return jwksCache, nil
	}

	kf, err := keyfunc.NewDefault([]string{url}, keyfunc.WithContext(context.Background()))
	if err != nil {
		return nil, err
	}

	jwksCache = kf
	jwksURL = url
	jwksExp = time.Now().Add(ttl)

	return kf, nil
}
