# Nucleus Android SDK (Kotlin)

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Android (Kotlin + Jetpack Compose).

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

MIT
