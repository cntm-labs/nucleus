# @cntm-labs/nextjs

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for Next.js.

## Installation

```bash
npm i @cntm-labs/nextjs@0.1.0-dev.1
```

## Quick Start

```tsx
// app/layout.tsx
import { NucleusProvider } from '@cntm-labs/nextjs';

export default function RootLayout({ children }) {
  return (
    <NucleusProvider publishableKey="pk_...">
      {children}
    </NucleusProvider>
  );
}
```

## License

MIT
