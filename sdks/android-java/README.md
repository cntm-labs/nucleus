<div align="center">

# Nucleus Android SDK (Java)

**Nucleus authentication SDK for Android (Java).**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-java?label=nucleus-java&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-java)

[![Java](https://img.shields.io/badge/Java-1.1k_LOC-ED8B00?logo=openjdk&logoColor=white)](src/)

[![Java](https://img.shields.io/badge/Java-ED8B00?logo=openjdk&logoColor=white)](https://openjdk.org/)
[![Android](https://img.shields.io/badge/Android-34A853?logo=android&logoColor=white)](https://developer.android.com/)

</div>

---

Manage sessions and user state in your Android app.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```groovy
implementation 'io.github.cntm-labs:nucleus-java:0.1.0'
```

## Quick Start

```java
Nucleus nucleus = Nucleus.configure(context, "pk_...");
Session session = nucleus.getSession();
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
