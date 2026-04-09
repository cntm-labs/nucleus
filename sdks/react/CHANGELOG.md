# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/@cntm-labs/react-v0.1.0...@cntm-labs/react-v0.2.0) (2026-04-09)


### Features

* add package registry publishing for all 13 SDKs ([6bc3b77](https://github.com/cntm-labs/nucleus/commit/6bc3b779c250c66da22fc16ede5de8c8e4f5deef))
* add Phase 8+9 SDKs (Go, Swift, Android, React, .NET, Rust) ([1523c56](https://github.com/cntm-labs/nucleus/commit/1523c569fd1f03c91c89952994be2c1a38e83b46))
* add Zod-based input validation for email and password across SDKs ([9b95dd1](https://github.com/cntm-labs/nucleus/commit/9b95dd120fc44e834900399988af21162785619e))
* implement 5 incomplete SDKs — full Clerk parity ([f9d1cc5](https://github.com/cntm-labs/nucleus/commit/f9d1cc5e3316334776da6e28276be5326473d656))
* Phase 5 — missing features for OAuth ([#8](https://github.com/cntm-labs/nucleus/issues/8)) ([8f65e14](https://github.com/cntm-labs/nucleus/commit/8f65e149d85c12f61971e034595b708a2b7b24de))
* **react:** add OAuth, passkey, MFA, verification, profile, and session hooks ([71da002](https://github.com/cntm-labs/nucleus/commit/71da002020bf0f405a846dc3183994bcf766169d))
* **react:** implement UI components with appearance theming ([9925ff5](https://github.com/cntm-labs/nucleus/commit/9925ff52865804976e4e5e1176d0d03c8c54721c))
* **sdks:** add runtime dev preview warnings to all SDKs ([ddc8908](https://github.com/cntm-labs/nucleus/commit/ddc890848e54cc0b7b2043fa1a7d0bb058188574))


### Bug Fixes

* address code review issues across all SDKs ([c3c8e63](https://github.com/cntm-labs/nucleus/commit/c3c8e6380e06e7a96abfeab0c16b8ce6c75385d9))
* **react,nextjs:** wire MFA TOTP verification in SignIn component ([4c5acef](https://github.com/cntm-labs/nucleus/commit/4c5acef75a9f93f64faabde87c242bc26c4dbac6))
* **react:** add OAuth CSRF state parameter ([a399899](https://github.com/cntm-labs/nucleus/commit/a3998996797e30bdf66ffff77460798a8d309237))
* **react:** fix session restore, add token refresh and org loading ([bc3101c](https://github.com/cntm-labs/nucleus/commit/bc3101cca27e10eb6eebd788883f710da542ee68))
* use Zod v4 error function API instead of deprecated required_error ([bb067d3](https://github.com/cntm-labs/nucleus/commit/bb067d39f0b0d1d06d0623cf5555033807162519))
