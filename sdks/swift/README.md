<div align="center">

# CntmNucleus

**Nucleus authentication SDK for iOS/macOS.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![CocoaPods](https://img.shields.io/cocoapods/v/CntmNucleus?label=CntmNucleus&color=EE3322)](https://cocoapods.org/pods/CntmNucleus)

[![Swift](https://img.shields.io/badge/Swift-1.2k_LOC-F05138?logo=swift&logoColor=white)](Sources/)

[![Swift](https://img.shields.io/badge/Swift-F05138?logo=swift&logoColor=white)](https://www.swift.org/)
[![Apple](https://img.shields.io/badge/iOS%20%7C%20macOS-000000?logo=apple&logoColor=white)](https://developer.apple.com/)

</div>

---

Manage sessions and user state in your Swift app.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

### Swift Package Manager

Add to your `Package.swift`:

```swift
.package(url: "https://github.com/cntm-labs/nucleus.git", from: "0.1.0")
```

### CocoaPods

```ruby
pod 'CntmNucleus', '~> 0.1.0'
```

## Quick Start

```swift
import CntmNucleus

let nucleus = Nucleus(publishableKey: "pk_...")
let session = try await nucleus.getSession()
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
