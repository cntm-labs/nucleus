"""Tests for nucleus.verify — token verification with JWKS."""

import pytest
from unittest.mock import patch
from datetime import datetime, timezone, timedelta
from cryptography.hazmat.primitives.asymmetric import rsa

import jwt as pyjwt

from nucleus.verify import verify_token, _get_jwks
from nucleus.claims import NucleusClaims
from .conftest import make_token, valid_claims


class TestVerifyTokenSuccess:
    def test_returns_nucleus_claims(self, jwks_response):
        token = make_token(valid_claims())
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            claims = verify_token(token, base_url="https://test.local")

        assert isinstance(claims, NucleusClaims)
        assert claims.user_id == "user_123"
        assert claims.project_id == "project_456"
        assert claims.email == "test@example.com"
        assert claims.first_name == "Test"
        assert claims.last_name == "User"

    def test_maps_org_claims(self, jwks_response):
        token = make_token(valid_claims(
            org_id="org_1",
            org_slug="my-org",
            org_role="admin",
            org_permissions=["read", "write"],
        ))
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            claims = verify_token(token, base_url="https://test.local")

        assert claims.org_id == "org_1"
        assert claims.org_slug == "my-org"
        assert claims.org_role == "admin"
        assert claims.permissions == ["read", "write"]


class TestVerifyTokenFailures:
    def test_expired_token_raises(self, jwks_response):
        expired = valid_claims(
            exp=int((datetime.now(timezone.utc) - timedelta(hours=1)).timestamp())
        )
        token = make_token(expired)
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            with pytest.raises(pyjwt.ExpiredSignatureError):
                verify_token(token, base_url="https://test.local")

    def test_wrong_key_raises(self, jwks_response):
        wrong_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
        token = pyjwt.encode(
            valid_claims(), wrong_key, algorithm="RS256", headers={"kid": "test-key-1"}
        )
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            with pytest.raises(pyjwt.InvalidSignatureError):
                verify_token(token, base_url="https://test.local")

    def test_missing_kid_raises(self, jwks_response):
        token = make_token(valid_claims(), kid="nonexistent-kid")
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            with pytest.raises(ValueError, match="No matching key"):
                verify_token(token, base_url="https://test.local")

    def test_invalid_token_string_raises(self, jwks_response):
        _get_jwks.cache_clear()
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            with pytest.raises(Exception):
                verify_token("not.a.valid.token", base_url="https://test.local")
