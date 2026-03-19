use chrono::{DateTime, Utc};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
pub struct MockClock {
    pub now: DateTime<Utc>,
}

#[cfg(test)]
impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        self.now
    }
}
