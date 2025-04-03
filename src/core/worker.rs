use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{
    activity::{AbstractActivityHandler, ActivityName},
    client::Client,
    workflow::{AbstractWorkflowHandler, ActivityOptions, WorkflowContext, WorkflowName},
};

#[derive(Clone)]
pub struct Worker {
    pub workflow_handlers: Arc<RwLock<HashMap<WorkflowName, Box<dyn AbstractWorkflowHandler>>>>,
    pub activity_handlers: Arc<RwLock<HashMap<ActivityName, Box<dyn AbstractActivityHandler>>>>,
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

    pub async fn register_workflow<W>(&mut self, workflow_handler: W) -> &Self
    where
        W: AbstractWorkflowHandler + 'static,
    {
        let name = WorkflowName::from(&workflow_handler);
        println!("Registered Workflow: {name}");
        let mut handlers = self.workflow_handlers.write().await;
        let _res = self.client.register_workflow(name.clone()).await;
        handlers.insert(name.clone(), Box::new(workflow_handler));
        self
    }

    pub async fn register_activity<H>(&mut self, activity_handler: H) -> &Self
    where
        H: AbstractActivityHandler + 'static,
    {
        let name = ActivityName::from(&activity_handler);
        println!("Registered Activity: {name}");
        let mut handlers = self.activity_handlers.write().await;
        handlers.insert(name.clone(), Box::new(activity_handler));
        let _ = self.client.register_activity(name.clone()).await;
        self
    }

    pub async fn poll_and_process_workflow(&self, name: WorkflowName) -> Result<String, String> {
        if let Some(poll_res) = self.client.poll_workflow(name.clone()).await? {
            let context = WorkflowContext {
                run_id: poll_res.workflow_run_id,
                activity_handlers: self.activity_handlers.clone(),
                client: self.client.clone(),
                event_count_order: 0,
                activity_options: ActivityOptions {
                    ..Default::default()
                },
            };

            if let Some(workflow_handler) = self.workflow_handlers.read().await.get(&poll_res.name)
            {
                let workflow_handler_result = workflow_handler.run(context, poll_res.input).await;

                match workflow_handler_result {
                    Ok(workflow_handler_result) => {
                        let _ = self
                            .client
                            .complete_workflow(
                                poll_res.workflow_id,
                                poll_res.workflow_run_id,
                                poll_res.rerun_of_workflow_run_id,
                                workflow_handler_result,
                                "".to_string(),
                            )
                            .await;
                    }
                    Err(workflow_handler_result) => {
                        let _ = self
                            .client
                            .complete_workflow(
                                poll_res.workflow_id,
                                poll_res.workflow_run_id,
                                poll_res.rerun_of_workflow_run_id,
                                "".to_string(),
                                workflow_handler_result,
                            )
                            .await;
                    }
                }
            }
        }

        Ok("done".to_string())
    }

    pub async fn poll_and_process_activity(&self, name: ActivityName) -> Result<String, String> {
        if let Some(poll_res) = self.client.poll_activity(name.clone()).await? {
            if let Some(activity_handler) = self.activity_handlers.read().await.get(&poll_res.name)
            {
                let activity_handler_result = activity_handler.run(poll_res.input).await;
                match activity_handler_result {
                    Ok(result) => {
                        let _ = self
                            .client
                            .complete_activity(
                                poll_res.activity_id,
                                poll_res.activity_run_id,
                                poll_res.workflow_run_id,
                                result,
                                "".to_string(),
                                poll_res.max_attempts,
                                poll_res.attempt_number,
                            )
                            .await;
                    }
                    Err(err) => {
                        let _ = self
                            .client
                            .complete_activity(
                                poll_res.activity_id,
                                poll_res.activity_run_id,
                                poll_res.workflow_run_id,
                                "".to_string(),
                                err,
                                poll_res.max_attempts,
                                poll_res.attempt_number,
                            )
                            .await;
                    }
                }
            }
        }

        Ok("done".to_string())
    }

    async fn run_workflows(&self) {
        for wf_name in self.workflow_handlers.read().await.keys() {
            {
                let wf_name = wf_name.clone();
                let worker = self.clone();
                tokio::task::spawn(async move {
                    loop {
                        let _ = worker.poll_and_process_workflow(wf_name.clone()).await;
                    }
                });
            }
        }
    }

    async fn run_activities(&self) {
        for act_name in self.activity_handlers.read().await.keys() {
            {
                let act_name = act_name.clone();
                let worker = self.clone();
                tokio::task::spawn(async move {
                    loop {
                        let _ = worker.poll_and_process_activity(act_name.clone()).await;
                    }
                });
            }
        }
    }

    pub async fn run(&self) {
        println!("Starting worker");
        {
            let worker = self.clone();
            tokio::task::spawn(async move { worker.run_workflows().await });
        }
        {
            let worker = self.clone();
            tokio::task::spawn(async move { worker.run_activities().await });
        }
    }

    pub async fn execute_workflow<W>(
        &mut self,
        workflow: W,
        input: String,
    ) -> Result<String, String>
    where
        W: AbstractWorkflowHandler + 'static,
    {
        let name = WorkflowName::from(&workflow);
        println!("Executing Workflow: {name}");

        let run_id = self.client.execute_workflow(name, input).await?;
        loop {
            if let Some(res) = self.client.poll_workflow_completion(run_id).await? {
                if !res.result.is_empty() {
                    return Ok(res.result);
                } else if !res.error.is_empty() {
                    return Err(res.error);
                }
            };
        }
    }
}
