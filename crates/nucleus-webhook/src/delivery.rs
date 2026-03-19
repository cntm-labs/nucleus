/// Delivery configuration
pub struct DeliveryConfig {
    pub max_attempts: u32,
    pub initial_delay_secs: u64,
    pub max_delay_secs: u64,
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_secs: 60,
            max_delay_secs: 3600,
        }
    }
}

impl DeliveryConfig {
    /// Calculate delay for attempt N using exponential backoff.
    ///
    /// - attempt 1: 60s
    /// - attempt 2: 120s
    /// - attempt 3: 240s
    /// - attempt 4: 480s
    /// - attempt 5: 960s (capped at max_delay_secs)
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        let delay = self.initial_delay_secs * 2u64.pow(attempt.saturating_sub(1));
        delay.min(self.max_delay_secs)
    }
}

/// Delivery status
#[derive(Debug, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    /// Max attempts reached
    Exhausted,
}

/// Delivery result
pub struct DeliveryResult {
    pub status: DeliveryStatus,
    pub response_code: Option<u16>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = DeliveryConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_secs, 60);
        assert_eq!(config.max_delay_secs, 3600);
    }

    #[test]
    fn exponential_backoff_calculation() {
        let config = DeliveryConfig::default();
        assert_eq!(config.delay_for_attempt(1), 60);
        assert_eq!(config.delay_for_attempt(2), 120);
        assert_eq!(config.delay_for_attempt(3), 240);
        assert_eq!(config.delay_for_attempt(4), 480);
        assert_eq!(config.delay_for_attempt(5), 960);
    }

    #[test]
    fn backoff_capped_at_max() {
        let config = DeliveryConfig {
            max_attempts: 10,
            initial_delay_secs: 60,
            max_delay_secs: 500,
        };
        // attempt 4 would be 480, attempt 5 would be 960 -> capped at 500
        assert_eq!(config.delay_for_attempt(4), 480);
        assert_eq!(config.delay_for_attempt(5), 500);
        assert_eq!(config.delay_for_attempt(10), 500);
    }
}
