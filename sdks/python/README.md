<div align="center">

# cntm-nucleus

**Nucleus authentication SDK for Python.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![PyPI](https://img.shields.io/pypi/v/cntm-nucleus?label=cntm-nucleus&color=3775A9)](https://pypi.org/project/cntm-nucleus/)

[![Python](https://img.shields.io/badge/Python-0.4k_LOC-3775A9?logo=python&logoColor=white)](src/)

[![Python](https://img.shields.io/badge/Python-3775A9?logo=python&logoColor=white)](https://www.python.org/)

</div>

---

Verify sessions and manage users from your Python backend.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
pip install cntm-nucleus
```

## Quick Start

```python
from nucleus import NucleusClient

client = NucleusClient(secret_key="sk_...")
session = client.verify_session(token)
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
