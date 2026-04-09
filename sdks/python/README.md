# cntm-nucleus

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Python.

## Installation

```bash
pip install cntm-nucleus==0.1.0
```

## Quick Start

```python
from nucleus import NucleusClient

client = NucleusClient(secret_key="sk_...")
session = client.verify_session(token)
```

## License

MIT
