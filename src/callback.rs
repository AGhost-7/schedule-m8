
use std::hash::{Hasher, Hash};
use std::cmp::{Ord, Ordering};
use std::time::{Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use cron::Schedule;
use chrono::Utc;
use std::str::FromStr;
use std::convert::TryFrom;
use crate::error::AppError;

// This type is the internal structure used by the scheduler.
#[derive(Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Callback {
    pub url: String,
    pub body: String,
    pub timestamp: Duration,
    pub id: String,
    pub schedule: Option<String>
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
        self.id.hash(state);
    }
}

// Data format for the v1 api.
#[derive(Deserialize, Serialize, Debug)]
pub struct V1Callback {
    pub timestamp: u64,
    pub url: String,
    pub payload: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct V1CronCallback {
    pub schedule: String,
    pub payload: String,
    pub name: String,
    pub group: String,
    pub url: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct V1CallbackKey {
    pub key: String
}

impl V1CallbackKey {
    pub fn new(key: String) -> V1CallbackKey {
        V1CallbackKey { key }
    }
}

impl TryFrom<V1CronCallback> for Callback {
    type Error = AppError;
    fn try_from(v1: V1CronCallback) -> Result<Callback, AppError> {
        let schedule = Schedule::from_str(&v1.schedule).map_err(|_|
            AppError::ValidationError
        )?;
        let id = v1.group.clone() + "::_" + &v1.name;
        let timestamp = schedule.upcoming(Utc).next().unwrap().timestamp();
        Ok(
            Callback {
                timestamp: Duration::from_millis(timestamp as u64),
                url: v1.url,
                body: v1.payload,
                id,
                schedule: Some(v1.schedule)
            }
        )
    }
}

impl From<V1Callback> for Callback {
    fn from(v1: V1Callback) -> Callback {
        let id = Uuid::new_v4().to_string();
        Callback {
            timestamp: Duration::from_millis(v1.timestamp),
            url: v1.url + "?key=" + &id,
            body: v1.payload,
            id,
            schedule: None
        }
    }
}
