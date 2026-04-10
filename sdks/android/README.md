<div align="center">

# Nucleus Android SDK (Kotlin)

**Nucleus authentication SDK for Android (Kotlin + Jetpack Compose).**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![Maven Central](https://img.shields.io/maven-central/v/io.github.cntm-labs/nucleus-android?label=nucleus-android&color=C71A36)](https://central.sonatype.com/artifact/io.github.cntm-labs/nucleus-android)

[![Kotlin](https://img.shields.io/badge/Kotlin-1.4k_LOC-7F52FF?logo=kotlin&logoColor=white)](src/)

[![Kotlin](https://img.shields.io/badge/Kotlin-7F52FF?logo=kotlin&logoColor=white)](https://kotlinlang.org/)
[![Android](https://img.shields.io/badge/Android-34A853?logo=android&logoColor=white)](https://developer.android.com/)
[![Jetpack Compose](https://img.shields.io/badge/Jetpack_Compose-4285F4?logo=jetpackcompose&logoColor=white)](https://developer.android.com/compose)

</div>

---

Manage sessions and user state in your Android app.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```groovy
implementation 'io.github.cntm-labs:nucleus-android:0.1.0'
```

## Quick Start

```kotlin
val nucleus = Nucleus.configure(context, publishableKey = "pk_...")
val session = nucleus.getSession()
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
