"""Shared test fixtures for Nucleus Python SDK tests."""

import json
import pytest
from datetime import datetime, timezone, timedelta
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.primitives import serialization
import jwt as pyjwt

# Generate a test RSA key pair (reused across all tests in the session)
_private_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
_public_key = _private_key.public_key()


@pytest.fixture
def private_key():
    return _private_key


@pytest.fixture
def public_key():
    return _public_key


@pytest.fixture
def jwks_response():
    """Build a JWKS JSON response from the test public key."""
    pub_numbers = _public_key.public_numbers()
    # Build JWK from public key components
    import base64

    def _int_to_base64url(n: int, length: int) -> str:
        return base64.urlsafe_b64encode(n.to_bytes(length, "big")).rstrip(b"=").decode()

    n = _int_to_base64url(pub_numbers.n, 256)  # 2048 bits = 256 bytes
    e = _int_to_base64url(pub_numbers.e, 3)

    return {
        "keys": [
            {
                "kty": "RSA",
                "kid": "test-key-1",
                "alg": "RS256",
                "use": "sig",
                "n": n,
                "e": e,
            }
        ]
    }


def make_token(claims: dict, kid: str = "test-key-1") -> str:
    """Sign a JWT with the test private key."""
    return pyjwt.encode(claims, _private_key, algorithm="RS256", headers={"kid": kid})


def valid_claims(**overrides) -> dict:
    """Build a valid set of JWT claims."""
    now = datetime.now(timezone.utc)
    claims = {
        "sub": "user_123",
        "iss": "https://api.test.com",
        "aud": "project_456",
        "exp": int((now + timedelta(hours=1)).timestamp()),
        "iat": int(now.timestamp()),
        "jti": "jwt_abc",
        "email": "test@example.com",
        "first_name": "Test",
        "last_name": "User",
    }
    claims.update(overrides)
    return claims
