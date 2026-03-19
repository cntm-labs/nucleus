import httpx
from .claims import NucleusClaims
from .verify import verify_token

class NucleusClient:
    def __init__(self, secret_key: str, base_url: str = "https://api.nucleus.dev"):
        self.secret_key = secret_key
        self.base_url = base_url
        self._client = httpx.AsyncClient(base_url=f"{base_url}/api/v1/admin",
            headers={"Authorization": f"Bearer {secret_key}", "Content-Type": "application/json"})
        self.users = UsersApi(self._client)
        self.orgs = OrgsApi(self._client)

    def verify_token(self, token: str) -> NucleusClaims:
        return verify_token(token, self.base_url)

    async def close(self):
        await self._client.aclose()

class UsersApi:
    def __init__(self, client: httpx.AsyncClient): self._client = client
    async def get(self, user_id: str): return (await self._client.get(f"/users/{user_id}")).json()
    async def list(self, limit: int = 20, cursor: str | None = None, email_contains: str | None = None):
        params = {"limit": limit}
        if cursor: params["cursor"] = cursor
        if email_contains: params["email_contains"] = email_contains
        return (await self._client.get("/users", params=params)).json()
    async def ban(self, user_id: str): await self._client.post(f"/users/{user_id}/ban")
    async def unban(self, user_id: str): await self._client.post(f"/users/{user_id}/unban")

class OrgsApi:
    def __init__(self, client: httpx.AsyncClient): self._client = client
    async def get(self, org_id: str): return (await self._client.get(f"/orgs/{org_id}")).json()
    async def list(self, limit: int = 20): return (await self._client.get("/orgs", params={"limit": limit})).json()
