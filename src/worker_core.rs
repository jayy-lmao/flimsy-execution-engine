use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    activity_core::AbstractActivityHandler,
    client::Client,
    workflow_core::{AbstractWorkflowHandler, WorkflowContext},
};

pub struct Worker {
    pub workflow_handlers: Arc<RwLock<HashMap<String, Box<dyn AbstractWorkflowHandler>>>>,
    pub activity_handlers: Arc<RwLock<HashMap<String, Box<dyn AbstractActivityHandler>>>>,
    client: Client,
}

impl Worker {
    pub fn new(client: Client) -> Self {
        Worker {
            workflow_handlers: Arc::new(RwLock::new(HashMap::new())),
            activity_handlers: Arc::new(RwLock::new(HashMap::new())),
            client,
        }
    }

    pub async fn register_workflow(
        &mut self,
        name: impl Into<String>,
        handler: impl AbstractWorkflowHandler + 'static,
    ) -> &Self {
        let name: String = name.into();
        let mut handlers = self.workflow_handlers.write().await;
        handlers.insert(name.clone(), Box::new(handler));
        self
    }

    pub async fn register_activity(
        &mut self,
        name: impl Into<String>,
        handler: impl AbstractActivityHandler + 'static,
    ) -> &Self {
        let name: String = name.into();
        let mut handlers = self.activity_handlers.write().await;
        handlers.insert(name.clone(), Box::new(handler));
        let _ = self.client.register_workflow(name.clone()).await;
        self
    }

    pub async fn execute_workflow(
        &mut self,
        workflow_name: impl Into<String>,
        input: String,
    ) -> Result<String, String> {
        let name: String = workflow_name.into();
        let workflow_run_id = Uuid::new_v4();
        let context = WorkflowContext {
            workflow_run_id,
            activity_handlers: self.activity_handlers.clone(),
        };

        if let Some(workflow_handler) = self.workflow_handlers.read().await.get(&name) {
            workflow_handler.run(context, input).await
        } else {
            Ok("".to_string())
        }
    }
}
