<div align="center">

# @cntm-labs/nucleus-node

**Nucleus authentication SDK for Node.js backends.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-node?label=@cntm-labs/nucleus-node&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-node)

[![TypeScript](https://img.shields.io/badge/TypeScript-0.3k_LOC-3178C6?logo=typescript&logoColor=white)](src/)

[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Node.js](https://img.shields.io/badge/Node.js-5FA04E?logo=nodedotjs&logoColor=white)](https://nodejs.org/)

</div>

---

Verify sessions, manage users, and protect API routes.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
npm i @cntm-labs/nucleus-node
```

## Quick Start

```typescript
import { createNucleus } from '@cntm-labs/nucleus-node';

const nucleus = createNucleus({ secretKey: 'sk_...' });
const { userId } = await nucleus.verifySession(token);
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
