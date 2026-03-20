use criterion::{criterion_group, criterion_main, Criterion};
use nucleus_auth::jwt::{JwtService, NucleusClaims};
use nucleus_core::types::*;

fn build_test_claims() -> NucleusClaims {
    JwtService::build_claims(
        &UserId::new(),
        &ProjectId::new(),
        "https://nucleus.bench",
        300,
        Some("bench@example.com".to_string()),
        Some("Bench".to_string()),
        Some("User".to_string()),
        None,
    )
}

fn bench_jwt_sign(c: &mut Criterion) {
    let key_pair = JwtService::generate_key_pair("bench-kid").unwrap();
    let claims = build_test_claims();
    c.bench_function("jwt_rs256_sign", |b| {
        b.iter(|| JwtService::sign(&claims, &key_pair))
    });
}

fn bench_jwt_verify(c: &mut Criterion) {
    let key_pair = JwtService::generate_key_pair("bench-kid").unwrap();
    let claims = build_test_claims();
    let token = JwtService::sign(&claims, &key_pair).unwrap();
    c.bench_function("jwt_rs256_verify", |b| {
        b.iter(|| JwtService::verify(&token, &key_pair.public_key_pem))
    });
}

fn bench_jwt_sign_and_verify(c: &mut Criterion) {
    let key_pair = JwtService::generate_key_pair("bench-kid").unwrap();
    c.bench_function("jwt_rs256_sign_and_verify", |b| {
        b.iter(|| {
            let claims = build_test_claims();
            let token = JwtService::sign(&claims, &key_pair).unwrap();
            JwtService::verify(&token, &key_pair.public_key_pem).unwrap();
        })
    });
}

fn bench_jwt_key_generation(c: &mut Criterion) {
    c.bench_function("jwt_rsa2048_keygen", |b| {
        b.iter(|| JwtService::generate_key_pair("bench-kid"))
    });
}

criterion_group!(benches, bench_jwt_sign, bench_jwt_verify, bench_jwt_sign_and_verify, bench_jwt_key_generation);
criterion_main!(benches);
