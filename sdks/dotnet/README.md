# Cntm.Nucleus

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for .NET.

## Installation

```bash
dotnet add package Cntm.Nucleus --version 0.1.0
```

## Quick Start

```csharp
using Nucleus;

var client = new NucleusClient("sk_...");
var session = await client.VerifySessionAsync(token);
```

## License

MIT
