# cntm-nucleus

> **Warning: DEV PREVIEW** — This crate is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Rust.

## Installation

```bash
cargo add cntm-nucleus@0.1.0
```

## Quick Start

```rust
use cntm_nucleus::NucleusClient;

let client = NucleusClient::new("sk_...");
let claims = client.verify_session(&token).await?;
```

## License

MIT
