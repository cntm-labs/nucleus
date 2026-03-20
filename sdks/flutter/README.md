# nucleus_flutter

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Flutter.

## Installation

```bash
flutter pub add nucleus_flutter:0.1.0-dev.1
```

## Quick Start

```dart
import 'package:nucleus_flutter/nucleus_flutter.dart';

final nucleus = NucleusClient(publishableKey: 'pk_...');
final session = await nucleus.getSession();
```

## License

MIT
