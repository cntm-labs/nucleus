from dataclasses import dataclass, field
from typing import Any

@dataclass
class NucleusClaims:
    user_id: str
    project_id: str
    email: str | None = None
    first_name: str | None = None
    last_name: str | None = None
    avatar_url: str | None = None
    email_verified: bool | None = None
    metadata: dict[str, Any] = field(default_factory=dict)
    org_id: str | None = None
    org_slug: str | None = None
    org_role: str | None = None
    permissions: list[str] = field(default_factory=list)
