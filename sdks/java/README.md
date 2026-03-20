# nucleus-java

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Java/Spring Boot.

## Installation

### Maven

```xml
<dependency>
  <groupId>io.github.cntm-labs</groupId>
  <artifactId>nucleus-java</artifactId>
  <version>0.1.0-dev.1</version>
</dependency>
```

### Gradle

```groovy
implementation 'io.github.cntm-labs:nucleus-java:0.1.0-dev.1'
```

## Quick Start

```java
NucleusClient client = new NucleusClient("sk_...");
Session session = client.verifySession(token);
```

## License

MIT
