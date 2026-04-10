<div align="center">

# nucleus

**Nucleus authentication SDK for Java/Spring Boot.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus?label=nucleus&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus)

[![Java](https://img.shields.io/badge/Java-0.6k_LOC-ED8B00?logo=openjdk&logoColor=white)](src/)

[![Java](https://img.shields.io/badge/Java-ED8B00?logo=openjdk&logoColor=white)](https://openjdk.org/)

</div>

---

Verify sessions and manage users from your Java backend.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

### Maven

```xml
<dependency>
  <groupId>io.github.cntm-labs</groupId>
  <artifactId>nucleus</artifactId>
  <version>0.1.0</version>
</dependency>
```

### Gradle

```groovy
implementation 'io.github.cntm-labs:nucleus:0.1.0'
```

## Quick Start

```java
NucleusClient client = new NucleusClient("sk_...");
Session session = client.verifySession(token);
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
