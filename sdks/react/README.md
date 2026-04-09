# @cntm-labs/nucleus-react

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for React SPAs.

## Installation

```bash
npm i @cntm-labs/nucleus-react@0.1.0
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

MIT
