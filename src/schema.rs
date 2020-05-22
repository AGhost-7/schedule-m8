
use std::hash::{Hasher, Hash};
use std::cmp::{Ord, Ordering};
use std::time::{Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::Utc;
use std::convert::TryFrom;
use crate::error::AppError;
use std::str::FromStr;

// This type is the internal structure used by the scheduler.
#[derive(Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub method: String,
    pub url: String,
    pub body: String,
    pub timestamp: Duration,
    pub id: String,
    pub schedule: Option<String>
}

impl Ord for Job {
    fn cmp(&self, other: &Job) -> Ordering {
        other.timestamp.cmp(&self.timestamp)
    }
}

impl PartialOrd for Job {
    fn partial_cmp(&self, other: &Job) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Job) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Data format for the v1 api.
#[derive(Deserialize, Serialize, Debug)]
pub struct V1Job {
    pub timestamp: u64,
    pub url: String,
    pub payload: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct V1CronJob {
    pub schedule: String,
    pub payload: String,
    pub name: String,
    pub group: String,
    pub url: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct V1JobKey {
    pub key: String
}

impl V1JobKey {
    pub fn new(key: String) -> V1JobKey {
        V1JobKey { key }
    }
}

impl TryFrom<V1CronJob> for Job {
    type Error = AppError;
    fn try_from(v1: V1CronJob) -> Result<Job, AppError> {
        // replace non-standard "?" with just a "*"
        let schedule_pattern = v1.schedule.replace("?", "*");
        let timestamp = cron::Schedule::from_str(&schedule_pattern)
            .map_err(|_| AppError::ValidationError)?
            .upcoming(Utc)
            .next()
            .ok_or(AppError::ValidationError)?;

        let id = v1.group.clone() + "::_" + &v1.name;
        Ok(
            Job {
                method: "POST".to_owned(),
                timestamp: Duration::from_millis(timestamp.timestamp_millis() as u64),
                url: v1.url,
                body: v1.payload,
                id,
                schedule: Some(schedule_pattern.to_owned())
            }
        )
    }
}

impl From<V1Job> for Job {
    fn from(v1: V1Job) -> Job {
        let id = Uuid::new_v4().to_string();
        Job {
            method: "POST".to_owned(),
            timestamp: Duration::from_millis(v1.timestamp),
            url: v1.url + "?key=" + &id,
            body: v1.payload,
            id,
            schedule: None
        }
    }
}

#[derive(Deserialize)]
pub struct V2Job {
    pub method: Option<String>,
    pub url: String,
    pub body: String,
    pub timestamp: u64
}

#[derive(Serialize)]
pub struct V2JobResponse <'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub body: &'a str,
    pub timestamp: u64,
    pub id: &'a str
}

#[derive(Deserialize)]
pub struct V2CronJob {
    pub method: Option<String>,
    pub url: String,
    pub body: String,
    pub schedule: String
}

#[derive(Serialize)]
pub struct V2CronJobResponse<'a> {
    pub id: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub body: &'a str,
    pub schedule: &'a str
}

impl TryFrom<V2Job> for Job {
    type Error = AppError;

    fn try_from(v2: V2Job) -> Result<Job, AppError> {
        let method = v2.method.unwrap_or_else(|| "POST".to_owned());
        hyper::Method::from_bytes(&method.as_bytes())
            .map_err(|_| AppError::ValidationError)?;

        Ok(
            Job {
                method: method,
                timestamp: Duration::from_millis(v2.timestamp),
                body: v2.body,
                url: v2.url,
                id: Uuid::new_v4().to_string(),
                schedule: None
            }
        )
    }
}

impl <'a> From<&'a Job> for V2JobResponse<'a> {
    fn from(job: &'a Job) -> V2JobResponse {
        V2JobResponse {
            id: &job.id,
            method: &job.method,
            url: &job.url,
            body: &job.body,
            timestamp: job.timestamp.as_millis() as u64,
        }
    }
}

impl TryFrom<V2CronJob> for Job {
    type Error = AppError;

    fn try_from(v2: V2CronJob) -> Result<Job, AppError> {
        let method = v2.method.unwrap_or_else(|| "POST".to_owned());
        hyper::Method::from_bytes(&method.as_bytes())
            .map_err(|_| AppError::ValidationError)?;

        let timestamp = cron::Schedule::from_str(&v2.schedule)
            .map_err(|_| AppError::ValidationError)?
            .upcoming(Utc)
            .next()
            .ok_or(AppError::ValidationError)?
            .timestamp_millis();
        Ok(
            Job {
                method: method,
                timestamp: Duration::from_millis(timestamp as u64),
                body: v2.body,
                url: v2.url,
                id: Uuid::new_v4().to_string(),
                schedule: None
            }
        )
    }
}

impl <'a> From<&'a Job> for V2CronJobResponse<'a> {
    fn from(job: &'a Job) -> V2CronJobResponse {
        V2CronJobResponse {
            id: &job.id,
            method: &job.method,
            url: &job.url,
            body: &job.body,
            schedule: &job.schedule.as_ref().unwrap()
        }
    }
}

#[test]
fn cron_deserialize() {
    let body = r#"{
        "payload": "",
        "schedule": "0 0 4 * * *",
        "group": "something",
        "name": "something",
        "url": "http://example.com/callback"
    }"#;
    let v1_cron: V1CronJob = serde_json::from_str(body).unwrap();
    let _job: Job = Job::try_from(v1_cron).unwrap();
}

#[test]
fn cron_year_deserialize() {
    let body = r#"{
        "payload": "",
        "schedule": " 0 0 7 * * ? *",
        "group": "something",
        "name": "something",
        "url": "http://example.com/callback"
    }"#;

    let v1_cron: V1CronJob = serde_json::from_str(body).unwrap();
    let _job = Job::try_from(v1_cron).unwrap();
}
