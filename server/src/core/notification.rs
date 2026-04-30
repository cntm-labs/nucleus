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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::Mutex;

    pub(crate) struct MockNotificationService {
        emails: Mutex<Vec<(String, String, String, String)>>,
        sms: Mutex<Vec<(String, String)>>,
    }

    impl MockNotificationService {
        pub(crate) fn new() -> Self {
            Self {
                emails: Mutex::new(Vec::new()),
                sms: Mutex::new(Vec::new()),
            }
        }

        pub(crate) fn sent_emails(&self) -> Vec<(String, String, String, String)> {
            self.emails.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl NotificationService for MockNotificationService {
        async fn send_email(
            &self,
            to: &str,
            subject: &str,
            html_body: &str,
            text_body: &str,
        ) -> Result<(), AppError> {
            self.emails.lock().unwrap().push((
                to.to_string(),
                subject.to_string(),
                html_body.to_string(),
                text_body.to_string(),
            ));
            Ok(())
        }

        async fn send_sms(&self, to: &str, body: &str) -> Result<(), AppError> {
            self.sms
                .lock()
                .unwrap()
                .push((to.to_string(), body.to_string()));
            Ok(())
        }
    }
}
