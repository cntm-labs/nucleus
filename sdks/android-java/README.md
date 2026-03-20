# Nucleus Android SDK (Java)

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Android (Java).

## Installation

```groovy
implementation 'io.github.cntm-labs:nucleus-android-java:0.1.0-dev.1'
```

## Quick Start

```java
Nucleus nucleus = Nucleus.configure(context, "pk_...");
Session session = nucleus.getSession();
```

## License

MIT
