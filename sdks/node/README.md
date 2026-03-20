# @cntm-labs/node

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Node.js backends.

## Installation

```bash
npm i @cntm-labs/node@0.1.0-dev.1
```

## Quick Start

```typescript
import { createNucleus } from '@cntm-labs/node';

const nucleus = createNucleus({ secretKey: 'sk_...' });
const { userId } = await nucleus.verifySession(token);
```

## License

MIT
