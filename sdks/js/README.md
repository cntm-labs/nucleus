<div align="center">

# @cntm-labs/nucleus-js

**Nucleus authentication SDK for vanilla JavaScript.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-js?label=@cntm-labs/nucleus-js&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-js)

[![TypeScript](https://img.shields.io/badge/TypeScript-1.4k_LOC-3178C6?logo=typescript&logoColor=white)](src/)

[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

</div>

---

Manage sessions and user state in any browser environment.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
npm i @cntm-labs/nucleus-js
```

## Quick Start

```javascript
import { Nucleus } from '@cntm-labs/nucleus-js';

const nucleus = new Nucleus({ publishableKey: 'pk_...' });
const session = await nucleus.getSession();
console.log('User:', session.userId);
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
