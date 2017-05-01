
use std::cmp::{Ord, Ordering};
use std::time::{Instant};

#[derive(Eq, Clone, Debug)]
pub struct Callback {
    pub url: String,
    pub body: String,
    pub timestamp: Instant
}

impl Ord for Callback {
    fn cmp(&self, other: &Callback) -> Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

impl PartialOrd for Callback {
    fn partial_cmp(&self, other: &Callback) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Callback {
    fn eq(&self, other: &Callback) -> bool {
        self.timestamp == other.timestamp
    }
}

