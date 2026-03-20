use criterion::{criterion_group, criterion_main, Criterion};

fn bench_password_hash(c: &mut Criterion) {
    c.bench_function("argon2id_hash", |b| {
        b.iter(|| nucleus_core::crypto::hash_password("test_password_123"))
    });
}

fn bench_password_verify(c: &mut Criterion) {
    let hash = nucleus_core::crypto::hash_password("test_password_123").unwrap();
    c.bench_function("argon2id_verify", |b| {
        b.iter(|| nucleus_core::crypto::verify_password("test_password_123", &hash))
    });
}

fn bench_token_generation(c: &mut Criterion) {
    c.bench_function("generate_token_256bit", |b| {
        b.iter(|| nucleus_core::crypto::generate_token())
    });
}

fn bench_hmac_sign(c: &mut Criterion) {
    let key = b"webhook_secret_key_here";
    let payload = b"event payload data for signing benchmark test";
    c.bench_function("hmac_sha256_sign", |b| {
        b.iter(|| nucleus_core::crypto::hmac_sign(key, payload))
    });
}

fn bench_aes_encrypt(c: &mut Criterion) {
    let key = nucleus_core::crypto::generate_encryption_key();
    let data = b"sensitive MFA secret data for encryption";
    c.bench_function("aes_256_gcm_encrypt", |b| {
        b.iter(|| nucleus_core::crypto::encrypt(data, &key))
    });
}

criterion_group!(
    benches,
    bench_password_hash,
    bench_password_verify,
    bench_token_generation,
    bench_hmac_sign,
    bench_aes_encrypt
);
criterion_main!(benches);
