<div align="center">

# cntm_nucleus

**Nucleus authentication SDK for Flutter.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![pub.dev](https://img.shields.io/pub/v/cntm_nucleus?label=cntm_nucleus&color=02569B)](https://pub.dev/packages/cntm_nucleus)

[![Dart](https://img.shields.io/badge/Dart-1.4k_LOC-02569B?logo=dart&logoColor=white)](lib/)

[![Flutter](https://img.shields.io/badge/Flutter-02569B?logo=flutter&logoColor=white)](https://flutter.dev/)
[![Dart](https://img.shields.io/badge/Dart-02569B?logo=dart&logoColor=white)](https://dart.dev/)

</div>

---

Manage sessions and user state in your Flutter app.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
flutter pub add cntm_nucleus
```

## Quick Start

```dart
import 'package:cntm_nucleus/cntm_nucleus.dart';

final nucleus = NucleusClient(publishableKey: 'pk_...');
final session = await nucleus.getSession();
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
