# cntm_nucleus

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Flutter.

## Installation

```bash
flutter pub add cntm_nucleus:0.1.0
```

## Quick Start

```dart
import 'package:cntm_nucleus/cntm_nucleus.dart';

final nucleus = NucleusClient(publishableKey: 'pk_...');
final session = await nucleus.getSession();
```

## License

MIT
