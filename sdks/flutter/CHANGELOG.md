# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/cntm-nucleus-flutter-v0.1.0...cntm-nucleus-flutter-v0.2.0) (2026-04-09)


### Features

* add package registry publishing for all 13 SDKs ([6bc3b77](https://github.com/cntm-labs/nucleus/commit/6bc3b779c250c66da22fc16ede5de8c8e4f5deef))
* add Phase 7 MVP SDKs (Node.js, Next.js, Flutter, Java, Python) ([9c426b7](https://github.com/cntm-labs/nucleus/commit/9c426b7820634389b18c3f63f43731d3225b2d9e))
* add Zod-based input validation for email and password across SDKs ([9b95dd1](https://github.com/cntm-labs/nucleus/commit/9b95dd120fc44e834900399988af21162785619e))
* **flutter:** implement full Nucleus SDK with Clerk parity ([f2a8090](https://github.com/cntm-labs/nucleus/commit/f2a80900f7389592cab2f49bc8ec88461f36832e))
* **flutter:** wire OAuth flow into NucleusAuth ([59ca703](https://github.com/cntm-labs/nucleus/commit/59ca7031dc3883901df82691881ca7bb165c2a9f))
* implement 5 incomplete SDKs — full Clerk parity ([f9d1cc5](https://github.com/cntm-labs/nucleus/commit/f9d1cc5e3316334776da6e28276be5326473d656))
* **sdks:** add runtime dev preview warnings to all SDKs ([ddc8908](https://github.com/cntm-labs/nucleus/commit/ddc890848e54cc0b7b2043fa1a7d0bb058188574))


### Bug Fixes

* address code review issues across all SDKs ([c3c8e63](https://github.com/cntm-labs/nucleus/commit/c3c8e6380e06e7a96abfeab0c16b8ce6c75385d9))
* **ci:** fix all SDK build issues for publish workflow ([af22bb4](https://github.com/cntm-labs/nucleus/commit/af22bb47220d09c00ccb827c7bee06ced5406e27))
* **flutter:** add OAuth CSRF state parameter ([52ac685](https://github.com/cntm-labs/nucleus/commit/52ac685cc6d157e04d605c6afa8686455db8c7d3))
* **flutter:** cache NucleusOAuth instance to preserve CSRF state ([87358c3](https://github.com/cntm-labs/nucleus/commit/87358c329928b71b3488d6ffcac5069b5a8743e7))
* **nextjs:** set refresh cookie expiry to 30 days instead of JWT expiry ([7e2688b](https://github.com/cntm-labs/nucleus/commit/7e2688be0959e163c9709b132f1e1c00bfed3e67))
* remove tracked build artifacts and resolve Flutter SDK errors ([#3](https://github.com/cntm-labs/nucleus/issues/3)) ([676f8d5](https://github.com/cntm-labs/nucleus/commit/676f8d58004c6ea78b6355f182a3f73f9028e111))
* update all remaining old package name references ([#40](https://github.com/cntm-labs/nucleus/issues/40)) ([6b47ea9](https://github.com/cntm-labs/nucleus/commit/6b47ea9589c00cb0a6280d580a1343c9ef2d14fb))

## 0.1.0

- Initial dev preview release
- Full authentication flows (signIn, signUp, signOut)
- OAuth support via url_launcher
- MFA (TOTP, SMS, backup codes)
- Organization RBAC
- UI widgets with theming
