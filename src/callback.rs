
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
#[derive(Deserialize, Serialize, Debug)]
pub struct V1Callback {
    pub timestamp: u64,
    pub url: String,
    pub payload: String
}

impl From<V1Callback> for Callback {
    fn from(v1: V1Callback) -> Callback {
        Callback {
            timestamp: Duration::from_millis(v1.timestamp),
            url: v1.url,
            body: v1.payload,
            uuid: Uuid::new_v4()
        }
    }
}
