use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::workflow_core::WorkflowRunId;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ActivityName(String);
impl<H> From<&H> for ActivityName
where
    H: AbstractActivityHandler,
{
    fn from(_value: &H) -> Self {
        let name_string: String = std::any::type_name::<H>()
            .rsplit("::")
            .next()
            .unwrap_or("")
            .to_string();
        ActivityName(name_string)
    }
}
impl fmt::Display for ActivityName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ActivityId(Uuid);
impl ActivityId {
    pub fn new() -> Self {
        ActivityId(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ActivityRunId(Uuid);
impl ActivityRunId {
    pub fn new() -> Self {
        ActivityRunId(Uuid::new_v4())
    }
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub id: ActivityId,
    pub name: ActivityName,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityEventType {
    Pending,
    Started,
    Succeeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ActivityEvent {
    pub activity_id: ActivityId,
    pub activity_run_id: ActivityRunId,
    pub workflow_run_id: WorkflowRunId,
    pub event_type: ActivityEventType,
    pub payload: String,
    pub created_at: DateTime<Utc>,
    pub attempt_number: i64,
    pub max_attempts: i64,
}

#[async_trait::async_trait]
pub trait AbstractActivityHandler: Send + Sync {
    async fn run(&self, input: String) -> Result<String, String>;
}
