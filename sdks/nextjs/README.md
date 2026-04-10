<div align="center">

# @cntm-labs/nucleus-nextjs

**Nucleus authentication SDK for Next.js.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-nextjs?label=@cntm-labs/nucleus-nextjs&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-nextjs)

[![TypeScript](https://img.shields.io/badge/TypeScript-2.3k_LOC-3178C6?logo=typescript&logoColor=white)](src/)

[![Next.js](https://img.shields.io/badge/Next.js-000000?logo=nextdotjs&logoColor=white)](https://nextjs.org/)
[![React](https://img.shields.io/badge/React-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

</div>

---

Works with App Router and Server Components.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
npm i @cntm-labs/nucleus-nextjs
```

## Quick Start

```tsx
// app/layout.tsx
import { NucleusProvider } from '@cntm-labs/nucleus-nextjs';

export default function RootLayout({ children }) {
  return (
    <NucleusProvider publishableKey="pk_...">
      {children}
    </NucleusProvider>
  );
}
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
