# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/@cntm-labs/nucleus-js-v0.1.0...@cntm-labs/nucleus-js-v0.2.0) (2026-04-09)


### Features

* add package registry publishing for all 13 SDKs ([6bc3b77](https://github.com/cntm-labs/nucleus/commit/6bc3b779c250c66da22fc16ede5de8c8e4f5deef))
* add Zod-based input validation for email and password across SDKs ([9b95dd1](https://github.com/cntm-labs/nucleus/commit/9b95dd120fc44e834900399988af21162785619e))
* implement 5 incomplete SDKs — full Clerk parity ([f9d1cc5](https://github.com/cntm-labs/nucleus/commit/f9d1cc5e3316334776da6e28276be5326473d656))
* **js:** add full API client and types for Clerk parity ([b880720](https://github.com/cntm-labs/nucleus/commit/b880720ce1854e1574304de152a7e5a18b4bcd87))
* **js:** add token storage and session manager with auto-refresh ([ef14cea](https://github.com/cntm-labs/nucleus/commit/ef14cea59b7ce2d4b0b76c49dd04932e816b37e9))
* **js:** implement full headless Nucleus client with Clerk parity ([6b6d2a7](https://github.com/cntm-labs/nucleus/commit/6b6d2a7ba70e149beebb9a590399ce3480f5832c))
* **sdk:** add nucleus-android-java and @nucleus/js (Phase 9 final) ([a51d754](https://github.com/cntm-labs/nucleus/commit/a51d754f44d23733c5a3e137541c3640888063ed))
* **sdks:** add runtime dev preview warnings to all SDKs ([ddc8908](https://github.com/cntm-labs/nucleus/commit/ddc890848e54cc0b7b2043fa1a7d0bb058188574))


### Bug Fixes

* **js:** add OAuth CSRF state parameter ([df76935](https://github.com/cntm-labs/nucleus/commit/df769357a6cf4408f9d66c8f9f46bdb95903ff95))
* **js:** add SSR guards for localStorage, window, navigator ([98e82dd](https://github.com/cntm-labs/nucleus/commit/98e82ddb13e41e82fa3af00a8131b8bd13d2a1c5))
* update all remaining old package name references ([#40](https://github.com/cntm-labs/nucleus/issues/40)) ([6b47ea9](https://github.com/cntm-labs/nucleus/commit/6b47ea9589c00cb0a6280d580a1343c9ef2d14fb))
* use Zod v4 error function API instead of deprecated required_error ([bb067d3](https://github.com/cntm-labs/nucleus/commit/bb067d39f0b0d1d06d0623cf5555033807162519))
