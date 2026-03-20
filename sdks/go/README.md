# nucleus-go

> **Warning: DEV PREVIEW** — This module is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Go.

## Installation

```bash
go get github.com/cntm-labs/nucleus/sdks/go@v0.1.0-dev.1
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

MIT
