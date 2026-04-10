use crate::core::error::AppError;
use crate::core::notification::NotificationService;
use async_trait::async_trait;
use std::sync::Arc;

/// Twilio-based SMS delivery service.
pub struct TwilioService {
    account_sid: String,
    auth_token: String,
    from_number: String,
    http_client: reqwest::Client,
}

impl TwilioService {
    pub fn new(account_sid: String, auth_token: String, from_number: String) -> Self {
        Self {
            account_sid,
            auth_token,
            from_number,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        let resp = self
            .http_client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&[("To", to), ("From", &self.from_number), ("Body", body)])
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Twilio error: {}",
                body
            )));
        }

        Ok(())
    }
}

/// Combines email (any NotificationService) + SMS (Twilio) into one service.
pub struct CompositeNotificationService {
    email: Arc<dyn NotificationService>,
    sms: Option<Arc<TwilioService>>,
}

impl CompositeNotificationService {
    pub fn new(email: Arc<dyn NotificationService>, sms: Option<Arc<TwilioService>>) -> Self {
        Self { email, sms }
    }
}

#[async_trait]
impl NotificationService for CompositeNotificationService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), AppError> {
        self.email
            .send_email(to, subject, html_body, text_body)
            .await
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        match &self.sms {
            Some(sms) => sms.send_sms(to, body).await,
            None => {
                tracing::warn!(to = %to, "SMS not configured — message not sent");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::email::LogNotificationService;

    #[tokio::test]
    async fn composite_delegates_email() {
        let email: Arc<dyn NotificationService> = Arc::new(LogNotificationService);
        let composite = CompositeNotificationService::new(email, None);
        let result = composite
            .send_email("user@test.com", "Hi", "<p>Hi</p>", "Hi")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn composite_sms_not_configured_succeeds() {
        let email: Arc<dyn NotificationService> = Arc::new(LogNotificationService);
        let composite = CompositeNotificationService::new(email, None);
        let result = composite.send_sms("+1234567890", "Hello").await;
        assert!(result.is_ok());
    }
}
