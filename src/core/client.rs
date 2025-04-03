use crate::core::{
    activity::{ActivityId, ActivityName, ActivityRunId},
    worker_events::{
        PollActivityCompletion, PollActivityResponse, PollWorkflowCompletion, PollWorkflowResponse,
        ServerEvent, WorkerEvent,
    },
    workflow::{WorkflowId, WorkflowName, WorkflowRunId},
};

#[derive(Clone)]
pub struct Client {
    pub client: reqwest::Client,
    pub base_url: String,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn register_workflow(&self, name: WorkflowName) -> Result<(), String> {
        let event = WorkerEvent::RegisterWorkflow { name };
        let _res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await;
        Ok(())
    }

    pub async fn register_activity(&self, name: ActivityName) -> Result<(), String> {
        let event = WorkerEvent::RegisterActivity { name };
        let _res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await;
        Ok(())
    }

    pub async fn execute_workflow(
        &mut self,
        name: WorkflowName,
        input: String,
    ) -> Result<WorkflowRunId, String> {
        let workflow_run_id = WorkflowRunId::new();

        let event = WorkerEvent::EnqueuWorkflow {
            name,
            input,
            workflow_run_id,
        };

        let _res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(workflow_run_id)
    }

    pub async fn execute_activity(
        &self,
        workflow_run_id: WorkflowRunId,
        name: ActivityName,
        input: String,
        max_attempts: i64,
    ) -> Result<ActivityRunId, String> {
        let activity_run_id = ActivityRunId::new();

        let event = WorkerEvent::EnqueuActivity {
            name,
            input,
            workflow_run_id,
            activity_run_id,
            max_attempts,
        };

        let _res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(activity_run_id)
    }

    pub async fn poll_workflow_completion(
        &self,
        workflow_run_id: WorkflowRunId,
    ) -> Result<Option<PollWorkflowCompletion>, String> {
        let event = WorkerEvent::PollWorkflowCompletion { workflow_run_id };

        let text_res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let server_event =
            serde_json::from_str::<ServerEvent>(&text_res).map_err(|e| e.to_string())?;

        match server_event {
            ServerEvent::PollWorkflowCompletion(poll_response) => Ok(Some(poll_response)),
            _ => Ok(None),
        }
    }

    pub async fn poll_workflow(
        &self,
        name: WorkflowName,
    ) -> Result<Option<PollWorkflowResponse>, String> {
        let event = WorkerEvent::PollWorkflow { name };

        let text_res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let server_event =
            serde_json::from_str::<ServerEvent>(&text_res).map_err(|e| e.to_string())?;

        match server_event {
            ServerEvent::PollWorkflowResponse(poll_response) => Ok(Some(poll_response)),
            _ => Ok(None),
        }
    }

    pub async fn complete_workflow(
        &self,
        workflow_id: WorkflowId,
        workflow_run_id: WorkflowRunId,
        rerun_of_workflow_run_id: Option<WorkflowRunId>,
        result: String,
        error: String,
    ) -> Result<String, String> {
        let event = WorkerEvent::CompleteWorkflow {
            result,
            error,
            workflow_id,
            workflow_run_id,
            rerun_of_workflow_run_id,
        };

        let res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(res)
    }

    pub async fn poll_activity_completion(
        &self,
        activity_run_id: ActivityRunId,
    ) -> Result<Option<PollActivityCompletion>, String> {
        let event = WorkerEvent::PollActivityCompletion { activity_run_id };

        let text_res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let server_event =
            serde_json::from_str::<ServerEvent>(&text_res).map_err(|e| e.to_string())?;

        match server_event {
            ServerEvent::PollActivityCompletion(poll_response) => Ok(Some(poll_response)),
            _ => Ok(None),
        }
    }

    pub async fn poll_activity(
        &self,
        name: ActivityName,
    ) -> Result<Option<PollActivityResponse>, String> {
        let event = WorkerEvent::PollActivity { name };

        let text_res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        let server_event =
            serde_json::from_str::<ServerEvent>(&text_res).map_err(|e| e.to_string())?;

        match server_event {
            ServerEvent::PollActivityResponse(poll_response) => Ok(Some(poll_response)),
            _ => Ok(None),
        }
    }

    pub async fn complete_activity(
        &self,
        activity_id: ActivityId,
        activity_run_id: ActivityRunId,
        workflow_run_id: WorkflowRunId,
        result: String,
        error: String,
        max_attempts: i64,
        attempt_number: i64,
    ) -> Result<String, String> {
        let event = WorkerEvent::CompleteActivity {
            result,
            error,
            activity_id,
            activity_run_id,
            workflow_run_id,
            max_attempts,
            attempt_number,
        };

        let res = self
            .client
            .post(format!("{}/worker_event", &self.base_url))
            .json(&event)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;

        Ok(res)
    }
}
