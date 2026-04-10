<div align="center">

# Cntm.Nucleus

**Nucleus authentication SDK for .NET.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![NuGet](https://img.shields.io/nuget/v/Cntm.Nucleus?label=Cntm.Nucleus&color=004880)](https://www.nuget.org/packages/Cntm.Nucleus)

[![C#](https://img.shields.io/badge/C%23-0.8k_LOC-512BD4?logo=dotnet&logoColor=white)](src/)

[![.NET](https://img.shields.io/badge/.NET-512BD4?logo=dotnet&logoColor=white)](https://dotnet.microsoft.com/)

</div>

---

Verify sessions and manage users from your .NET backend.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
dotnet add package Cntm.Nucleus
```

## Quick Start

```csharp
using Nucleus;

var client = new NucleusClient("sk_...");
var session = await client.VerifySessionAsync(token);
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
