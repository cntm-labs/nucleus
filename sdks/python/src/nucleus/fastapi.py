from fastapi import Depends, HTTPException, Request
from .claims import NucleusClaims
from .verify import verify_token

class NucleusAuth:
    def __init__(self, secret_key: str | None = None, base_url: str = "https://api.nucleus.dev"):
        self.base_url = base_url

    async def __call__(self, request: Request) -> NucleusClaims:
        auth = request.headers.get("authorization", "")
        if not auth.startswith("Bearer "):
            raise HTTPException(401, detail="Missing authorization header")
        try:
            return verify_token(auth[7:], self.base_url)
        except Exception:
            raise HTTPException(401, detail="Invalid or expired token")

def require_permission(permission: str):
    def decorator(func):
        from functools import wraps
        @wraps(func)
        async def wrapper(*args, claims: NucleusClaims = Depends(), **kwargs):
            if permission not in (claims.permissions or []) and claims.org_role != "owner":
                raise HTTPException(403, detail="Insufficient permissions")
            return await func(*args, claims=claims, **kwargs)
        return wrapper
    return decorator
