# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/cntm-nucleus-server-v0.1.0...cntm-nucleus-server-v0.2.0) (2026-04-09)


### Features

* add webhooks, dashboard API, JWKS, and federated template (Phase 5) ([2bd4b2d](https://github.com/cntm-labs/nucleus/commit/2bd4b2da227aab9d4c4e9db1f97cfd2806c5c47f))
* **auth:** add MfaEnrollmentRepository and wire MFA verify handler ([ae2e15d](https://github.com/cntm-labs/nucleus/commit/ae2e15db9e65c8920229973183b2ad7fd0327496))
* **auth:** wire magic link send and verify handlers ([881710a](https://github.com/cntm-labs/nucleus/commit/881710ad408a689899e932ffc59cb40503a21d0f))
* **auth:** wire OTP send and verify handlers with Redis storage ([d76700c](https://github.com/cntm-labs/nucleus/commit/d76700c37ffe4c160cae97a5a4cd6fbb690c5da5))
* **auth:** wire password reset request and confirm handlers ([028b9cc](https://github.com/cntm-labs/nucleus/commit/028b9cc78ca82d2ebb34f2e5271c9f56d8434048))
* complete Phases 10-11 — deployment, CI/CD, load tests, monitoring ([918d8c2](https://github.com/cntm-labs/nucleus/commit/918d8c2e9718ec5481c977c2f574274e03aa0f79))
* complete stub handlers — Phase 2 production readiness ([771cd1a](https://github.com/cntm-labs/nucleus/commit/771cd1abb10c6f35055d9026414c3d08140f9bdf))
* connect all handlers to services — zero todo!() remaining ([d45ed0b](https://github.com/cntm-labs/nucleus/commit/d45ed0b23bb189a8aa92517247db70a47e33591a))
* **dashboard:** implement projects, API keys, audit logs, and settings with real DB queries ([4437c0f](https://github.com/cntm-labs/nucleus/commit/4437c0f50e523326c7a0d4cc2d611bd641ada3bb))
* Phase 5 — missing features for OAuth ([#8](https://github.com/cntm-labs/nucleus/issues/8)) ([8f65e14](https://github.com/cntm-labs/nucleus/commit/8f65e149d85c12f61971e034595b708a2b7b24de))
* scaffold Cargo workspace with 10 crates ([4601c1c](https://github.com/cntm-labs/nucleus/commit/4601c1c32fc9c714d74c4fb027b60e952f56d08e))
* **server:** add config, state, router, health endpoint, and middleware ([4ac9c1b](https://github.com/cntm-labs/nucleus/commit/4ac9c1b11ff9d4fdce1e7a5065e4d74342ac95ab))
* **server:** add JWT/API key auth extractors and rate limiting ([e973c61](https://github.com/cntm-labs/nucleus/commit/e973c61e29103941fa2f343f7dcc02cc12df59d5))
* **server:** add uptime tracking to health endpoint ([7385817](https://github.com/cntm-labs/nucleus/commit/7385817b6ebadce3b997473eeb7aab34788bbb62))
* **server:** wire Phase 2 auth routes into Axum router ([68d95e6](https://github.com/cntm-labs/nucleus/commit/68d95e697beb5cba034d80d516f9d4aadb451a48))
* **server:** wire Phase 3 auth routes ([f974a07](https://github.com/cntm-labs/nucleus/commit/f974a07c220474ef38c5e44a6061f3ce6d640f64))
* **server:** wire Phase 4 identity, org, and admin routes ([f02ee89](https://github.com/cntm-labs/nucleus/commit/f02ee89473cbed389a7fc17eda4741c323e9c00d))
* **server:** wire Phase 5 — webhook admin and dashboard API routes ([118961f](https://github.com/cntm-labs/nucleus/commit/118961fd4a72795c1e975fcd2e1e265a633070d5))


### Bug Fixes

* **auth:** check JWT revocation list in auth middleware ([0aa68cb](https://github.com/cntm-labs/nucleus/commit/0aa68cb32e9479703101d3d587d59fd3f4571a8f))
* **auth:** require JWT authentication for sign-out and refresh endpoints ([abbee84](https://github.com/cntm-labs/nucleus/commit/abbee8412eee39fc06f7fa1de8c946bc7f8f4824))
* **ci:** add license to all crates, fix deny.toml advisories, regenerate lockfile ([0f79186](https://github.com/cntm-labs/nucleus/commit/0f7918680d1e694238239b1b479dc7fa68000c2a))
* **ci:** fix wildcard deps and regenerate lockfile for Node 22 ([6c2a32a](https://github.com/cntm-labs/nucleus/commit/6c2a32a9c4fdb8567632352e7893f010de772973))
* **org:** require JWT authentication for all organization routes ([cbfae0b](https://github.com/cntm-labs/nucleus/commit/cbfae0bdd4241dc54dc8e3a9ef5cc48dc5618e3a))
* Phase 1 security blockers for production readiness ([be885da](https://github.com/cntm-labs/nucleus/commit/be885da2f847e8adfdd2b3bb2c3083cfaf7c8ae9))
* **security:** comprehensive security hardening before public release ([#4](https://github.com/cntm-labs/nucleus/issues/4)) ([5c919ef](https://github.com/cntm-labs/nucleus/commit/5c919ef2c05c7a9df4e83e03a626003940624dfe))
* **server:** eliminate all dead code — wire middleware into router ([abd13ec](https://github.com/cntm-labs/nucleus/commit/abd13ec3be5a03ff3bcebe3503813eadc40b0b27))
* **server:** persist JWT signing keys in database with AES-256-GCM encryption ([201aef3](https://github.com/cntm-labs/nucleus/commit/201aef317e0d53ba763dd77182f033bf4584d95d))
* **server:** use Axum 0.8 path syntax {param} instead of :param ([025c10a](https://github.com/cntm-labs/nucleus/commit/025c10a8dfd7564fe8c99e9d23cabc4459ca755a))
