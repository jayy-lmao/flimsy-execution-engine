use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::activity_core::{AbstractActivityHandler, Activity, ActivityEvent, ActivityEventType};

pub struct Workflow {
    pub workflow_id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub enum WorkflowEventType {
    Pending,
    Started,
    Succeeeded,
    Failed,
}

#[derive(Debug)]
pub struct WorkflowEvent {
    pub workflow_id: Uuid,
    pub workflow_run_id: Uuid,
    pub workflow_event_type: WorkflowEventType,
    pub payload: String,
}

pub struct WorkflowContext {
    pub workflow_run_id: Uuid,

    pub activity_handlers: Arc<RwLock<HashMap<String, Box<dyn AbstractActivityHandler>>>>,
    // pub activities: Arc<RwLock<Vec<Activity>>>,
    // pub activity_events: Arc<RwLock<Vec<ActivityEvent>>>,
}

impl WorkflowContext {
    pub async fn execute_activity(
        &self,
        activity_name: impl Into<String>,
        input: String,
    ) -> Result<String, String> {
        let name: String = activity_name.into();
        let activity_run_id = Uuid::new_v4();
        // if let Some(activity) = self.activities.read().await.iter().find(|w| w.name == name) {
        //     self.activity_events.write().await.push(ActivityEvent {
        //         activity_id: activity.activity_id,
        //         activity_run_id,
        //         workflow_run_id: self.workflow_run_id,
        //         activity_event_type: ActivityEventType::Started,
        //         payload: input.clone(),
        //     });
        // }

        if let Some(activity_handler) = self.activity_handlers.read().await.get(&name) {
            let result = activity_handler.run(input).await;

            // if let Some(activity) = self.activities.read().await.iter().find(|w| w.name == name) {
            //     match result {
            //         Ok(ref payload) => self.activity_events.write().await.push(ActivityEvent {
            //             activity_id: activity.activity_id,
            //             activity_run_id,
            //             workflow_run_id: self.workflow_run_id,
            //             activity_event_type: ActivityEventType::Succeeeded,
            //             payload: payload.clone(),
            //         }),
            //         Err(ref payload) => self.activity_events.write().await.push(ActivityEvent {
            //             activity_id: activity.activity_id,
            //             activity_run_id,
            //             workflow_run_id: self.workflow_run_id,
            //             activity_event_type: ActivityEventType::Failed,
            //             payload: payload.clone(),
            //         }),
            //     };
            // }
            result
        } else {
            Ok("".to_string())
        }
    }
}

#[async_trait::async_trait]
pub trait AbstractWorkflowHandler: Send + Sync {
    async fn run(&self, context: WorkflowContext, input: String) -> Result<String, String>;
}
