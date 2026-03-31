package nucleus

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestProtect_ValidToken(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-mw")
	defer server.Close()

	cfg := &Config{BaseURL: server.URL}

	var capturedClaims *NucleusClaims
	handler := Protect(cfg)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		capturedClaims = ClaimsFromContext(r.Context())
		w.WriteHeader(http.StatusOK)
	}))

	token := makeToken(t, key, validClaims(), "test-key-mw")
	req := httptest.NewRequest("GET", "/protected", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rr.Code)
	}
	if capturedClaims == nil {
		t.Fatal("expected claims in context")
	}
	if capturedClaims.UserID() != "user_123" {
		t.Errorf("expected user_123, got %s", capturedClaims.UserID())
	}
}

func TestProtect_MissingToken(t *testing.T) {
	cfg := &Config{SecretKey: "sk_test"}

	handler := Protect(cfg)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("handler should not be called without token")
	}))

	req := httptest.NewRequest("GET", "/protected", nil)
	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d", rr.Code)
	}
}

func TestProtect_InvalidToken(t *testing.T) {
	resetJWKSCache()
	key := generateTestKey(t)
	server := startJWKSServer(t, key, "test-key-inv")
	defer server.Close()

	cfg := &Config{BaseURL: server.URL}

	handler := Protect(cfg)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("handler should not be called with invalid token")
	}))

	req := httptest.NewRequest("GET", "/protected", nil)
	req.Header.Set("Authorization", "Bearer invalid.token.here")
	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if rr.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d", rr.Code)
	}
}

func TestClaimsFromContext_WithClaims(t *testing.T) {
	claims := &NucleusClaims{Email: "test@example.com"}
	ctx := context.WithValue(context.Background(), contextKey{}, claims)
	result := ClaimsFromContext(ctx)
	if result == nil {
		t.Fatal("expected claims from context")
	}
	if result.Email != "test@example.com" {
		t.Errorf("expected test@example.com, got %s", result.Email)
	}
}

func TestClaimsFromContext_Empty(t *testing.T) {
	result := ClaimsFromContext(context.Background())
	if result != nil {
		t.Error("expected nil claims from empty context")
	}
}
