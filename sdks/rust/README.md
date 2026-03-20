# nucleus-rs

> **Warning: DEV PREVIEW** — This crate is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Rust.

## Installation

```bash
cargo add nucleus-rs@0.1.0-dev.1
```

## Quick Start

```rust
use nucleus_rs::NucleusClient;

let client = NucleusClient::new("sk_...");
let claims = client.verify_session(&token).await?;
```

## License

MIT
