
use std::hash::{Hasher, Hash};
use std::cmp::{Ord, Ordering};
use std::time::{Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// This type is the internal structure used by the scheduler.
#[derive(Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Callback {
    pub url: String,
    pub body: String,
    pub timestamp: Duration,
    pub uuid: Uuid
}

pub trait Schedulable {
    fn to_schedulable(self) -> Callback;
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

impl Hash for Callback {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}

// Data format for the v1 api.
#[derive(Deserialize, Serialize)]
pub struct V1Callback {
    pub timestamp: u64,
    pub url: String,
    pub payload: String
}

impl Schedulable for V1Callback {
    fn to_schedulable(self) -> Callback {
        Callback {
            url: self.url,
            body: self.payload,
            timestamp: Duration::from_millis(self.timestamp),
            uuid: Uuid::new_v4()
        }
    }
}
