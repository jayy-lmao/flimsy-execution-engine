use serde::{Deserialize, Serialize};

use crate::core::{
    activity::{ActivityId, ActivityName, ActivityRunId},
    workflow::{WorkflowId, WorkflowName, WorkflowRunId},
};

#[derive(Serialize, Deserialize)]
pub enum WorkerEvent {
    RegisterWorkflow {
        name: WorkflowName,
    },
    RegisterActivity {
        name: ActivityName,
    },
    EnqueuWorkflow {
        name: WorkflowName,
        input: String,
        workflow_run_id: WorkflowRunId,
    },
    EnqueuActivity {
        name: ActivityName,
        input: String,
        activity_run_id: ActivityRunId,
        workflow_run_id: WorkflowRunId,
        max_attempts: i64,
    },
    CompleteWorkflow {
        result: String,
        error: String,
        workflow_id: WorkflowId,
        workflow_run_id: WorkflowRunId,
        rerun_of_workflow_run_id: Option<WorkflowRunId>,
    },
    PollWorkflow {
        name: WorkflowName,
    },
    PollWorkflowCompletion {
        workflow_run_id: WorkflowRunId,
    },
    CompleteActivity {
        result: String,
        error: String,
        activity_id: ActivityId,
        activity_run_id: ActivityRunId,
        workflow_run_id: WorkflowRunId,
        max_attempts: i64,
        attempt_number: i64,
    },
    PollActivity {
        name: ActivityName,
    },
    PollActivityCompletion {
        activity_run_id: ActivityRunId,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PollWorkflowResponse {
    pub workflow_run_id: WorkflowRunId,
    pub rerun_of_workflow_run_id: Option<WorkflowRunId>,
    pub workflow_id: WorkflowId,
    pub name: WorkflowName,
    pub input: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PollActivityResponse {
    pub activity_run_id: ActivityRunId,
    pub workflow_run_id: WorkflowRunId,
    pub activity_id: ActivityId,
    pub name: ActivityName,
    pub input: String,
    pub max_attempts: i64,
    pub attempt_number: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PollWorkflowCompletion {
    pub workflow_run_id: WorkflowRunId,
    pub result: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PollActivityCompletion {
    pub activity_run_id: ActivityRunId,
    pub result: String,
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    PollWorkflowResponse(PollWorkflowResponse),
    PollActivityResponse(PollActivityResponse),
    PollWorkflowCompletion(PollWorkflowCompletion),
    PollActivityCompletion(PollActivityCompletion),
    GeneralSuccess { success: bool },
    NotFound,
}
