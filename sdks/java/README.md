# nucleus

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Java/Spring Boot.

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

MIT
