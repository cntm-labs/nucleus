import jwt
import httpx
from functools import lru_cache
from .claims import NucleusClaims

@lru_cache(maxsize=1)
def _get_jwks(base_url: str) -> dict:
    res = httpx.get(f"{base_url}/.well-known/jwks.json")
    res.raise_for_status()
    return res.json()

def verify_token(token: str, base_url: str = "https://api.nucleus.dev") -> NucleusClaims:
    jwks = _get_jwks(base_url)
    header = jwt.get_unverified_header(token)
    key = next((k for k in jwks["keys"] if k["kid"] == header.get("kid")), None)
    if not key:
        raise ValueError("No matching key found in JWKS")
    public_key = jwt.algorithms.RSAAlgorithm.from_jwk(key)
    payload = jwt.decode(token, public_key, algorithms=["RS256"], options={"verify_aud": False})
    return NucleusClaims(
        user_id=payload["sub"], project_id=payload.get("aud", ""),
        email=payload.get("email"), first_name=payload.get("first_name"),
        last_name=payload.get("last_name"), avatar_url=payload.get("avatar_url"),
        email_verified=payload.get("email_verified"), metadata=payload.get("metadata", {}),
        org_id=payload.get("org_id"), org_slug=payload.get("org_slug"),
        org_role=payload.get("org_role"), permissions=payload.get("org_permissions", []))
