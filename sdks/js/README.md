# @cntm-labs/js

> **Warning: DEV PREVIEW** — This package is under active development
> and is NOT ready for production use. APIs may change without notice.
> For updates, watch the [Nucleus repo](https://github.com/cntm-labs/nucleus).

Nucleus authentication SDK for vanilla JavaScript.

## Installation

```bash
npm i @cntm-labs/js@0.1.0-dev.1
```

## Quick Start

```javascript
import { Nucleus } from '@cntm-labs/js';

const nucleus = new Nucleus({ publishableKey: 'pk_...' });
const session = await nucleus.getSession();
```

## License

MIT
