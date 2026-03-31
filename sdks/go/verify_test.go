package nucleus

import (
	"crypto/rand"
	"crypto/rsa"
	"encoding/base64"
	"encoding/json"
	"math/big"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

func generateTestKey(t *testing.T) *rsa.PrivateKey {
	t.Helper()
	key, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		t.Fatal(err)
	}
	return key
}

func base64urlEncode(b []byte) string {
	return base64.RawURLEncoding.EncodeToString(b)
}

func buildJWKS(t *testing.T, key *rsa.PrivateKey, kid string) map[string]interface{} {
	t.Helper()
	pub := key.Public().(*rsa.PublicKey)
	return map[string]interface{}{
		"keys": []map[string]interface{}{
			{
				"kty": "RSA",
				"kid": kid,
				"alg": "RS256",
				"use": "sig",
				"n":   base64urlEncode(pub.N.Bytes()),
				"e":   base64urlEncode(big.NewInt(int64(pub.E)).Bytes()),
			},
		},
	}
}

func startJWKSServer(t *testing.T, key *rsa.PrivateKey, kid string) *httptest.Server {
	t.Helper()
	jwks := buildJWKS(t, key, kid)
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(jwks)
	}))
}

func makeToken(t *testing.T, key *rsa.PrivateKey, claims jwt.MapClaims, kid string) string {
	t.Helper()
	token := jwt.NewWithClaims(jwt.SigningMethodRS256, claims)
	token.Header["kid"] = kid
	signed, err := token.SignedString(key)
	if err != nil {
		t.Fatal(err)
	}
	return signed
}

func resetJWKSCache() {
	jwksMu.Lock()
	defer jwksMu.Unlock()
	jwksCache = nil
	jwksURL = ""
	jwksExp = time.Time{}
}

func validClaims() jwt.MapClaims {
	return jwt.MapClaims{
		"sub":        "user_123",
		"iss":        "https://api.test.com",
		"aud":        "project_456",
		"exp":        time.Now().Add(time.Hour).Unix(),
		"iat":        time.Now().Unix(),
		"jti":        "jwt_abc",
		"email":      "test@example.com",
		"first_name": "Test",
		"last_name":  "User",
	}
}

func TestVerifyToken_Valid(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-1")
	defer server.Close()

	token := makeToken(t, key, validClaims(), "test-key-1")
	cfg := &Config{BaseURL: server.URL}

	claims, err := VerifyToken(token, cfg)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if claims.Subject != "user_123" {
		t.Errorf("expected subject user_123, got %s", claims.Subject)
	}
	if claims.UserID() != "user_123" {
		t.Errorf("expected UserID() user_123, got %s", claims.UserID())
	}
	if claims.Email != "test@example.com" {
		t.Errorf("expected email test@example.com, got %s", claims.Email)
	}
	if claims.FirstName != "Test" {
		t.Errorf("expected first_name Test, got %s", claims.FirstName)
	}
}

func TestVerifyToken_Expired(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-1")
	defer server.Close()

	expired := validClaims()
	expired["exp"] = time.Now().Add(-time.Hour).Unix()
	token := makeToken(t, key, expired, "test-key-1")
	cfg := &Config{BaseURL: server.URL}

	_, err := VerifyToken(token, cfg)
	if err == nil {
		t.Fatal("expected error for expired token")
	}
}

func TestVerifyToken_WrongKey(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	wrongKey := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-1") // Serves key's public key
	defer server.Close()

	// Sign with wrongKey but use kid that maps to key's public key
	token := makeToken(t, wrongKey, validClaims(), "test-key-1")
	cfg := &Config{BaseURL: server.URL}

	_, err := VerifyToken(token, cfg)
	if err == nil {
		t.Fatal("expected error for wrong key")
	}
}

func TestVerifyToken_InvalidTokenString(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-1")
	defer server.Close()

	cfg := &Config{BaseURL: server.URL}
	_, err := VerifyToken("not.a.valid.token", cfg)
	if err == nil {
		t.Fatal("expected error for invalid token string")
	}
}

func TestClaimsFromContext_Nil(t *testing.T) {
	claims := ClaimsFromContext(nil)
	if claims != nil {
		t.Error("expected nil claims from nil context")
	}
}
