"""Tests for nucleus.client — NucleusClient initialization and delegation."""

import pytest
from unittest.mock import patch, AsyncMock
from nucleus.client import NucleusClient
from nucleus.claims import NucleusClaims
from .conftest import make_token, valid_claims


class TestNucleusClientInit:
    def test_default_base_url(self):
        client = NucleusClient(secret_key="sk_test")
        assert client.base_url == "https://api.nucleus.dev"

    def test_custom_base_url(self):
        client = NucleusClient(secret_key="sk_test", base_url="https://custom.api.dev")
        assert client.base_url == "https://custom.api.dev"

    def test_has_users_api(self):
        client = NucleusClient(secret_key="sk_test")
        assert client.users is not None

    def test_has_orgs_api(self):
        client = NucleusClient(secret_key="sk_test")
        assert client.orgs is not None


class TestNucleusClientVerifyToken:
    def test_delegates_to_verify_token(self, jwks_response):
        from nucleus.verify import _get_jwks
        _get_jwks.cache_clear()
        client = NucleusClient(secret_key="sk_test", base_url="https://test.local")
        token = make_token(valid_claims())
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            claims = client.verify_token(token)

        assert isinstance(claims, NucleusClaims)
        assert claims.user_id == "user_123"

    def test_forwards_audience_parameter(self, jwks_response):
        from nucleus.verify import _get_jwks
        _get_jwks.cache_clear()
        client = NucleusClient(secret_key="sk_test", base_url="https://test.local")
        token = make_token(valid_claims(aud="project_456"))
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            claims = client.verify_token(token, audience="project_456")

        assert claims.project_id == "project_456"

    def test_wrong_audience_via_client_raises(self, jwks_response):
        import jwt as pyjwt
        from nucleus.verify import _get_jwks
        _get_jwks.cache_clear()
        client = NucleusClient(secret_key="sk_test", base_url="https://test.local")
        token = make_token(valid_claims(aud="project_456"))
        with patch("nucleus.verify._get_jwks", return_value=jwks_response):
            with pytest.raises(pyjwt.InvalidAudienceError):
                client.verify_token(token, audience="wrong_project")


class TestNucleusClaims:
    def test_default_values(self):
        claims = NucleusClaims(user_id="u1", project_id="p1")
        assert claims.user_id == "u1"
        assert claims.project_id == "p1"
        assert claims.email is None
        assert claims.metadata == {}
        assert claims.permissions == []

    def test_all_fields(self):
        claims = NucleusClaims(
            user_id="u1", project_id="p1",
            email="test@example.com", first_name="Test", last_name="User",
            avatar_url="https://img.test/a.png", email_verified=True,
            metadata={"role": "admin"}, org_id="org_1", org_slug="my-org",
            org_role="admin", permissions=["read", "write"],
        )
        assert claims.email == "test@example.com"
        assert claims.email_verified is True
        assert claims.metadata == {"role": "admin"}
        assert claims.org_role == "admin"
        assert claims.permissions == ["read", "write"]
