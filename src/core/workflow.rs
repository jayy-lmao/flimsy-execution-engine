use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::activity::{AbstractActivityHandler, ActivityName};
use crate::core::client::Client;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct WorkflowName(String);
impl<H> From<&H> for WorkflowName
where
    H: AbstractWorkflowHandler,
{
    fn from(_value: &H) -> Self {
        let name_string: String = std::any::type_name::<H>()
            .rsplit("::")
            .next()
            .unwrap_or("")
            .to_string();

        WorkflowName(name_string)
    }
}
impl fmt::Display for WorkflowName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct WorkflowId(Uuid);
impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowId {
    pub fn new() -> Self {
        WorkflowId(Uuid::new_v4())
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct WorkflowRunId(Uuid);
impl Default for WorkflowRunId {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRunId {
    pub fn new() -> Self {
        WorkflowRunId(Uuid::new_v4())
    }
}
impl fmt::Display for WorkflowRunId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: WorkflowName,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WorkflowEventType {
    Pending,
    Started,
    Succeeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct WorkflowEvent {
    pub workflow_id: WorkflowId,
    pub run_id: WorkflowRunId,
    pub event_type: WorkflowEventType,
    pub payload: String,
    pub rerun_of: Option<WorkflowRunId>,
    pub created_at: DateTime<Utc>,
}

pub struct WorkflowContext {
    pub run_id: WorkflowRunId,
    pub event_count_order: i64,
    pub activity_handlers: Arc<RwLock<HashMap<ActivityName, Box<dyn AbstractActivityHandler>>>>,
    pub client: Client,
    pub activity_options: ActivityOptions,
}

pub struct RetryOptions {
    pub max_attempts: i64,
}

impl Default for RetryOptions {
    fn default() -> Self {
        Self { max_attempts: 1 }
    }
}

#[derive(Default)]
pub struct ActivityOptions {
    pub retry_policy: RetryOptions,
}

impl WorkflowContext {
    pub fn with_activity_options(&mut self, activity_options: ActivityOptions) {
        self.activity_options = activity_options;
    }
    pub async fn execute_activity<H>(&mut self, handler: H, input: String) -> Result<String, String>
    where
        H: AbstractActivityHandler + 'static,
    {
        let name = ActivityName::from(&handler);
        println!("Executing Activity: {name}");
        self.event_count_order += 1;
        let run_id = self
            .client
            .execute_activity(
                self.run_id,
                name,
                input,
                self.activity_options.retry_policy.max_attempts,
            )
            .await?;

        loop {
            if let Some(res) = self.client.poll_activity_completion(run_id).await? {
                if !res.result.is_empty() {
                    return Ok(res.result);
                } else if !res.error.is_empty() {
                    return Err(res.error);
                }
            };
        }
    }
}

#[async_trait::async_trait]
pub trait AbstractWorkflowHandler: Send + Sync {
    async fn run(&self, context: WorkflowContext, input: String) -> Result<String, String>;
}
