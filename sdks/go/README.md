<div align="center">

# cntm-nucleus-go

**Nucleus authentication SDK for Go.**

> **Warning: DEV PREVIEW** — This module is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![Go Reference](https://pkg.go.dev/badge/github.com/cntm-labs/nucleus/sdks/go.svg)](https://pkg.go.dev/github.com/cntm-labs/nucleus/sdks/go)

[![Go](https://img.shields.io/badge/Go-0.9k_LOC-00ADD8?logo=go&logoColor=white)](.)

[![Go](https://img.shields.io/badge/Go-00ADD8?logo=go&logoColor=white)](https://go.dev/)

</div>

---

Verify sessions and manage users from your Go backend.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
go get github.com/cntm-labs/nucleus/sdks/go
```

## Quick Start

```go
package main

import nucleus "github.com/cntm-labs/nucleus/sdks/go"

func main() {
    client := nucleus.NewClient("sk_...")
    claims, err := client.VerifySession(token)
}
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
