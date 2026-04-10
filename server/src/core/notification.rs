use crate::core::error::AppError;
use async_trait::async_trait;

/// Trait for sending notifications (email, SMS).
/// Implementations live in nucleus-server (composition root).
#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), AppError>;

    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError>;
}
