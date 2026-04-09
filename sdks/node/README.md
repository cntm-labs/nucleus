# @cntm-labs/nucleus-node

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Node.js backends.

## Installation

```bash
npm i @cntm-labs/nucleus-node@0.1.0
```

## Quick Start

```typescript
import { createNucleus } from '@cntm-labs/nucleus-node';

const nucleus = createNucleus({ secretKey: 'sk_...' });
const { userId } = await nucleus.verifySession(token);
```

## License

MIT
