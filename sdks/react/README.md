<div align="center">

# @cntm-labs/nucleus-react

**Nucleus authentication SDK for React SPAs.**

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

[![CI](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/ci.yml)
[![Security](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml/badge.svg)](https://github.com/cntm-labs/nucleus/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)

[![npm](https://img.shields.io/npm/v/@cntm-labs/nucleus-react?label=@cntm-labs/nucleus-react&color=cb3837)](https://www.npmjs.com/package/@cntm-labs/nucleus-react)

[![TypeScript](https://img.shields.io/badge/TypeScript-2.4k_LOC-3178C6?logo=typescript&logoColor=white)](src/)

[![React](https://img.shields.io/badge/React-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

</div>

---

Provider component and hooks for session and user state.

Part of [Nucleus](https://github.com/cntm-labs/nucleus) — high-performance, self-hosted auth platform.

## Installation

```bash
npm i @cntm-labs/nucleus-react
```

## Quick Start

```tsx
import { NucleusProvider, useAuth } from '@cntm-labs/nucleus-react';

function App() {
  return (
    <NucleusProvider publishableKey="pk_...">
      <MyApp />
    </NucleusProvider>
  );
}

function MyApp() {
  const { isSignedIn, user } = useAuth();
  return <div>{isSignedIn ? `Hello ${user.email}` : 'Sign in'}</div>;
}
```

## License

[MIT](https://github.com/cntm-labs/nucleus/blob/main/LICENSE)
