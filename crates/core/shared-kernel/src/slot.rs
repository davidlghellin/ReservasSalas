use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub inicio: DateTime<Utc>,
}

impl Slot {
    pub fn new(inicio: DateTime<Utc>) -> Self {
        Self { inicio }
    }

    pub fn fin(&self) -> DateTime<Utc> {
        self.inicio + Duration::hours(1)
    }
}