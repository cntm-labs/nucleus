import httpx
from .claims import NucleusClaims
from .verify import verify_token

class SyncNucleusClient:
    def __init__(self, secret_key: str, base_url: str = "https://api.nucleus.dev"):
        self.secret_key = secret_key
        self.base_url = base_url
        self._client = httpx.Client(base_url=f"{base_url}/api/v1/admin",
            headers={"Authorization": f"Bearer {secret_key}", "Content-Type": "application/json"})
        self.users = SyncUsersApi(self._client)

    def verify_token(self, token: str) -> NucleusClaims:
        return verify_token(token, self.base_url)

class SyncUsersApi:
    def __init__(self, client: httpx.Client): self._client = client
    def get(self, user_id: str): return self._client.get(f"/users/{user_id}").json()
    def list(self, limit: int = 20): return self._client.get("/users", params={"limit": limit}).json()
