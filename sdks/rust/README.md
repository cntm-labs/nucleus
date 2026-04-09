<div align="center">

# cntm-nucleus

**Nucleus authentication SDK for Rust.**

> **Warning: DEV PREVIEW** — This crate is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![crates.io](https://img.shields.io/crates/v/cntm-nucleus?label=cntm-nucleus&color=fc8d62)](https://crates.io/crates/cntm-nucleus)

[![Rust](https://img.shields.io/badge/Rust-0.9k_LOC-dea584?logo=rust&logoColor=white)](src/)

[![Rust](https://img.shields.io/badge/Rust-dea584?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-dea584?logo=rust&logoColor=white)](https://tokio.rs/)

</div>

---

Verify sessions and manage users from your Rust backend.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
cargo add cntm-nucleus
```

## Quick Start

```rust
use cntm_nucleus::NucleusClient;

let client = NucleusClient::new("sk_...");
let claims = client.verify_session(&token).await?;
println!("User ID: {}", claims.sub);
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
