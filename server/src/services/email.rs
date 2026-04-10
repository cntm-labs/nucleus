use crate::core::error::AppError;
use crate::core::notification::NotificationService;
use async_trait::async_trait;

/// SendGrid-based email delivery service.
pub struct SendGridService {
    api_key: String,
    from_email: String,
    from_name: String,
    http_client: reqwest::Client,
}

impl SendGridService {
    pub fn new(api_key: String, from_email: String, from_name: String) -> Self {
        Self {
            api_key,
            from_email,
            from_name,
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl NotificationService for SendGridService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), AppError> {
        let payload = serde_json::json!({
            "personalizations": [{"to": [{"email": to}]}],
            "from": {"email": &self.from_email, "name": &self.from_name},
            "subject": subject,
            "content": [
                {"type": "text/plain", "value": text_body},
                {"type": "text/html", "value": html_body},
            ]
        });

        let resp = self
            .http_client
            .post("https://api.sendgrid.com/v3/mail/send")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Internal(e.into()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "SendGrid error: {}",
                body
            )));
        }

        Ok(())
    }

    async fn send_sms(&self, _to: &str, _body: &str) -> Result<(), AppError> {
        Err(AppError::Internal(anyhow::anyhow!(
            "SMS not supported by SendGrid service"
        )))
    }
}

/// Fallback for development: logs emails instead of sending.
pub struct LogNotificationService;

#[async_trait]
impl NotificationService for LogNotificationService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        _html_body: &str,
        _text_body: &str,
    ) -> Result<(), AppError> {
        tracing::info!(to = %to, subject = %subject, "Email would be sent (dev mode)");
        Ok(())
    }

    async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
        tracing::info!(to = %to, body = %body, "SMS would be sent (dev mode)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn log_service_send_email_succeeds() {
        let service = LogNotificationService;
        let result = service
            .send_email("user@example.com", "Test", "<p>Hi</p>", "Hi")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn log_service_send_sms_succeeds() {
        let service = LogNotificationService;
        let result = service.send_sms("+1234567890", "Hello").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sendgrid_sms_returns_error() {
        let service = SendGridService::new(
            "fake-key".to_string(),
            "test@example.com".to_string(),
            "Test".to_string(),
        );
        let result = service.send_sms("+1234567890", "Hello").await;
        assert!(result.is_err());
    }
}
