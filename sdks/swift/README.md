# CntmNucleus

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for iOS/macOS.

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

MIT
