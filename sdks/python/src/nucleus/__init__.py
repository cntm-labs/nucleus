import warnings

__version__ = "0.1.0.dev1"
if "dev" in __version__:
    warnings.warn(
        f"[Nucleus] You are using a dev preview ({__version__}). Do not use in production.",
        stacklevel=2,
    )

from .client import NucleusClient
from .verify import verify_token
from .claims import NucleusClaims

__all__ = ["NucleusClient", "verify_token", "NucleusClaims"]
