# Changelog

## [0.2.0](https://github.com/cntm-labs/nucleus/compare/CntmNucleus-v0.1.0...CntmNucleus-v0.2.0) (2026-04-09)


### Features

* add package registry publishing for all 13 SDKs ([6bc3b77](https://github.com/cntm-labs/nucleus/commit/6bc3b779c250c66da22fc16ede5de8c8e4f5deef))
* add Phase 8+9 SDKs (Go, Swift, Android, React, .NET, Rust) ([1523c56](https://github.com/cntm-labs/nucleus/commit/1523c569fd1f03c91c89952994be2c1a38e83b46))
* **sdks:** add runtime dev preview warnings to all SDKs ([ddc8908](https://github.com/cntm-labs/nucleus/commit/ddc890848e54cc0b7b2043fa1a7d0bb058188574))


### Bug Fixes

* Android AGP singleVariant + Maven URL, CocoaPods tag creation ([#70](https://github.com/cntm-labs/nucleus/issues/70)) ([3e9524b](https://github.com/cntm-labs/nucleus/commit/3e9524b9ec6f69c8d44667ec2fa803cde2c43c2b))
* CocoaPods source_files path relative to repo root ([#71](https://github.com/cntm-labs/nucleus/issues/71)) ([832dac4](https://github.com/cntm-labs/nucleus/commit/832dac46ea6f216046dc34604ffed01a49d1495c))
* gradlew JVM opts quoting, Swift podspec tag, Go write permissions ([#69](https://github.com/cntm-labs/nucleus/issues/69)) ([142973c](https://github.com/cntm-labs/nucleus/commit/142973cf984519b91a7d421c76adc95c9014f88f))
* remove macOS target from podspec (UI code is iOS-only) ([#73](https://github.com/cntm-labs/nucleus/issues/73)) ([43d0c3a](https://github.com/cntm-labs/nucleus/commit/43d0c3a49ad8faf833be258db803e00a52fa58e9))
* resolve remaining SDK build failures for registry publish ([#62](https://github.com/cntm-labs/nucleus/issues/62)) ([9042feb](https://github.com/cntm-labs/nucleus/commit/9042febb25a8684a14d0aa4a3645cfbdf22aa49e))
* resolve SDK build failures blocking registry publish ([#60](https://github.com/cntm-labs/nucleus/issues/60)) ([18559e0](https://github.com/cntm-labs/nucleus/commit/18559e03f672e9013603f6021d677c8ae5cf5be8))
* Swift ForEach id, Android Maven Central repository config ([#67](https://github.com/cntm-labs/nucleus/issues/67)) ([521a6d5](https://github.com/cntm-labs/nucleus/commit/521a6d5df631e570c4dad1f9d9728709d2ee7a65))
* Swift type-check, Android Gradle wrapper, Go module tag format ([#68](https://github.com/cntm-labs/nucleus/issues/68)) ([d461ac8](https://github.com/cntm-labs/nucleus/commit/d461ac8311bb0035f139662d122bbbe59788c285))
* update all remaining old package name references ([#40](https://github.com/cntm-labs/nucleus/issues/40)) ([6b47ea9](https://github.com/cntm-labs/nucleus/commit/6b47ea9589c00cb0a6280d580a1343c9ef2d14fb))
