pub mod pool;
pub mod redis;
pub mod repos;

#[cfg(test)]
mod tests {
    #[test]
    fn repo_modules_compile() {
        // This test exists just to ensure all module imports work.
        // The use statements below verify the traits and types are accessible.
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<super::repos::PgProjectRepository>();
        _assert_send_sync::<super::repos::PgUserRepository>();
        _assert_send_sync::<super::repos::PgCredentialRepository>();
        _assert_send_sync::<super::repos::RedisSessionRepository>();
        _assert_send_sync::<super::repos::PgApiKeyRepository>();
        _assert_send_sync::<super::repos::PgAuditRepository>();
        _assert_send_sync::<super::repos::PgOrgRepository>();
        _assert_send_sync::<super::repos::PgWebhookRepository>();
    }
}
