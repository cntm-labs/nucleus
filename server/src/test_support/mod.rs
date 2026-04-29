//! Test-only support: shared constants, fixtures, and helpers used across
//! module-level test blocks. Gated on `#[cfg(test)]` so it is never built
//! into the release binary.

#[cfg(test)]
pub mod fixtures;
