# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/@cntm-labs/nucleus-nextjs-v0.1.0...@cntm-labs/nucleus-nextjs-v0.2.0) (2026-04-09)


### Features

* add package registry publishing for all 13 SDKs ([6bc3b77](https://github.com/cntm-labs/nucleus/commit/6bc3b779c250c66da22fc16ede5de8c8e4f5deef))
* add Phase 7 MVP SDKs (Node.js, Next.js, Flutter, Java, Python) ([9c426b7](https://github.com/cntm-labs/nucleus/commit/9c426b7820634389b18c3f63f43731d3225b2d9e))
* add Zod-based input validation for email and password across SDKs ([9b95dd1](https://github.com/cntm-labs/nucleus/commit/9b95dd120fc44e834900399988af21162785619e))
* implement 5 incomplete SDKs — full Clerk parity ([f9d1cc5](https://github.com/cntm-labs/nucleus/commit/f9d1cc5e3316334776da6e28276be5326473d656))
* **nextjs:** add full hook set and UI components ([0807bc8](https://github.com/cntm-labs/nucleus/commit/0807bc8117d20934c1de3bb21f746319acae5a42))
* **nextjs:** add server-side HttpOnly cookie setter via Server Action ([7488ebe](https://github.com/cntm-labs/nucleus/commit/7488ebe2e6039f984bb56f6cf21fafe9e99230fe))
* **nextjs:** implement server-side JWT verification and auth middleware ([d9fa95e](https://github.com/cntm-labs/nucleus/commit/d9fa95e518609669b18eaf25881caa382e6e8ab2))
* Phase 5 — missing features for OAuth ([#8](https://github.com/cntm-labs/nucleus/issues/8)) ([8f65e14](https://github.com/cntm-labs/nucleus/commit/8f65e149d85c12f61971e034595b708a2b7b24de))
* **sdks:** add runtime dev preview warnings to all SDKs ([ddc8908](https://github.com/cntm-labs/nucleus/commit/ddc890848e54cc0b7b2043fa1a7d0bb058188574))


### Bug Fixes

* address code review issues across all SDKs ([c3c8e63](https://github.com/cntm-labs/nucleus/commit/c3c8e6380e06e7a96abfeab0c16b8ce6c75385d9))
* **nextjs:** add OAuth CSRF state parameter ([59e70f7](https://github.com/cntm-labs/nucleus/commit/59e70f742ee31630973715bb4256497352621084))
* **nextjs:** set refresh cookie expiry to 30 days instead of JWT expiry ([7e2688b](https://github.com/cntm-labs/nucleus/commit/7e2688be0959e163c9709b132f1e1c00bfed3e67))
* **nextjs:** wire useSignIn/useSignUp hooks and session restore ([5e69894](https://github.com/cntm-labs/nucleus/commit/5e698942d2b7dbd00b9634042dd8ab64ce469b56))
* **react,nextjs:** wire MFA TOTP verification in SignIn component ([4c5acef](https://github.com/cntm-labs/nucleus/commit/4c5acef75a9f93f64faabde87c242bc26c4dbac6))
* update all remaining old package name references ([#40](https://github.com/cntm-labs/nucleus/issues/40)) ([6b47ea9](https://github.com/cntm-labs/nucleus/commit/6b47ea9589c00cb0a6280d580a1343c9ef2d14fb))
* use Zod v4 error function API instead of deprecated required_error ([bb067d3](https://github.com/cntm-labs/nucleus/commit/bb067d39f0b0d1d06d0623cf5555033807162519))
