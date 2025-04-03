use std::time::Duration;

use crate::core::activity::{Activity, ActivityEvent, ActivityEventType, ActivityId};
use crate::core::worker_events::{
    PollActivityCompletion, PollActivityResponse, PollWorkflowCompletion, PollWorkflowResponse,
    ServerEvent, WorkerEvent,
};
use crate::core::workflow::{
    Workflow, WorkflowEvent, WorkflowEventType, WorkflowId, WorkflowRunId,
};
use crate::inmemory_db::Db;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;

pub struct Server {
    state: ServerState,
}

#[derive(Clone)]
pub struct ServerState {
    db: Db,
}

#[derive(Deserialize)]
pub struct RerunWorkflowPayload {
    workflow_run_id: WorkflowRunId,
}

async fn handle_rerun_workflow(
    State(state): State<ServerState>,
    Json(payload): Json<RerunWorkflowPayload>,
) -> impl IntoResponse {
    let last_event = state
        .db
        .get_last_workflow_run_event(payload.workflow_run_id)
        .await;

    if let Some(last_event) = last_event {
        if let Some(first_event) = state
            .db
            .get_first_workflow_run_event(payload.workflow_run_id)
            .await
        {
            let new_workflow_run_id = WorkflowRunId::new();

            let event_type = last_event.event_type;
            if event_type == WorkflowEventType::Failed {
                state
                    .db
                    .add_workflow_event(WorkflowEvent {
                        workflow_id: last_event.workflow_id,
                        run_id: new_workflow_run_id,
                        event_type: WorkflowEventType::Pending,
                        rerun_of: Some(last_event.run_id),
                        payload: first_event.payload,
                        created_at: Utc::now(),
                    })
                    .await;
                return Json(json!({ "new_workflow_id": new_workflow_run_id }));
            }
        }
    }

    Json(json!({ "error": "workflow not found" }))
}

async fn handle_worker_event(
    State(state): State<ServerState>,
    Json(event): Json<WorkerEvent>,
) -> impl IntoResponse {
    let db = state.db;
    match event {
        WorkerEvent::RegisterWorkflow { name } => {
            let exists = db.workflow_exists(&name).await;
            if !exists {
                db.add_workflow(Workflow {
                    name,
                    id: WorkflowId::new(),
                })
                .await;
            }
        }
        WorkerEvent::EnqueuWorkflow {
            name,
            input,
            workflow_run_id,
        } => {
            if let Some(workflow) = db.get_workflow_by_name(&name).await {
                db.add_workflow_event(WorkflowEvent {
                    workflow_id: workflow.id,
                    run_id: workflow_run_id,
                    event_type: WorkflowEventType::Pending,
                    rerun_of: None,
                    payload: input,
                    created_at: Utc::now(),
                })
                .await;
            }
        }
        WorkerEvent::RegisterActivity { name } => {
            let existing = db.activity_exists(&name).await;
            if !existing {
                db.add_activity(Activity {
                    name,
                    id: ActivityId::new(),
                })
                .await
            }
            return Json(ServerEvent::GeneralSuccess { success: true });
        }
        WorkerEvent::EnqueuActivity {
            name,
            input,
            activity_run_id,
            workflow_run_id,
            max_attempts,
        } => {
            if let Some(workflow) = db.get_last_workflow_run_event(workflow_run_id).await {
                if let Some(activity) = db.get_activity_by_name(&name).await {
                    if let Some(past_workflow_run_id) = workflow.rerun_of {
                        if let Some(past_success_of_activity) = db
                            .get_success_activity_event_for_run(
                                past_workflow_run_id,
                                activity.id,
                                &input,
                            )
                            .await
                        {
                            if past_success_of_activity.payload == input {
                                db.add_activity_event(ActivityEvent {
                                    activity_id: activity.id,
                                    activity_run_id,
                                    workflow_run_id,
                                    event_type: ActivityEventType::Succeeeded,
                                    payload: past_success_of_activity.payload,
                                    created_at: Utc::now(),
                                    attempt_number: 1,
                                    max_attempts,
                                })
                                .await;

                                return Json(ServerEvent::GeneralSuccess { success: true });
                            }
                        }
                    }
                    db.add_activity_event(ActivityEvent {
                        activity_id: activity.id,
                        activity_run_id,
                        workflow_run_id,
                        event_type: ActivityEventType::Pending,
                        payload: input,
                        created_at: Utc::now(),
                        attempt_number: 1,
                        max_attempts,
                    })
                    .await;
                }
            }
        }
        WorkerEvent::PollWorkflow { name } => loop {
            if let Some(pending) = db.get_first_pending_workflow(name.clone()).await {
                db.add_workflow_event(WorkflowEvent {
                    workflow_id: pending.workflow_id,
                    run_id: pending.run_id,
                    event_type: WorkflowEventType::Started,
                    rerun_of: pending.rerun_of,
                    payload: "".to_string(),
                    created_at: Utc::now(),
                })
                .await;

                return Json(ServerEvent::PollWorkflowResponse(PollWorkflowResponse {
                    workflow_run_id: pending.run_id,
                    rerun_of_workflow_run_id: pending.rerun_of,
                    workflow_id: pending.workflow_id,
                    name,
                    input: pending.payload.clone(),
                }));
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        },
        WorkerEvent::CompleteWorkflow {
            result,
            error,
            workflow_id,
            workflow_run_id,
            rerun_of_workflow_run_id,
        } => {
            println!("Completed Workflow, RunId = {}\n", workflow_run_id);
            if error.is_empty() {
                db.add_workflow_event(WorkflowEvent {
                    workflow_id,
                    run_id: workflow_run_id,
                    event_type: WorkflowEventType::Succeeeded,
                    rerun_of: rerun_of_workflow_run_id,
                    payload: result,
                    created_at: Utc::now(),
                })
                .await;
            } else {
                db.add_workflow_event(WorkflowEvent {
                    workflow_id,
                    run_id: workflow_run_id,
                    event_type: WorkflowEventType::Failed,
                    rerun_of: rerun_of_workflow_run_id,
                    payload: error,
                    created_at: Utc::now(),
                })
                .await;
            }
        }
        WorkerEvent::PollWorkflowCompletion { workflow_run_id } => loop {
            if let Some(completed) = db.get_completed_workflow(workflow_run_id).await {
                match completed.event_type {
                    WorkflowEventType::Succeeeded => {
                        return Json(ServerEvent::PollWorkflowCompletion(
                            PollWorkflowCompletion {
                                workflow_run_id,
                                result: completed.payload,
                                error: "".to_string(),
                            },
                        ));
                    }
                    WorkflowEventType::Failed => {
                        return Json(ServerEvent::PollWorkflowCompletion(
                            PollWorkflowCompletion {
                                workflow_run_id,
                                error: completed.payload,
                                result: "".to_string(),
                            },
                        ));
                    }
                    _ => {}
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        },
        WorkerEvent::PollActivity { name } => loop {
            if let Some(pending) = db.get_first_pending_activity(name.clone()).await {
                let attempt_number = if pending.event_type == ActivityEventType::Pending {
                    pending.attempt_number
                } else {
                    pending.attempt_number + 1
                };

                db.add_activity_event(ActivityEvent {
                    workflow_run_id: pending.workflow_run_id,
                    payload: "".to_string(),
                    activity_id: pending.activity_id,
                    activity_run_id: pending.activity_run_id,
                    event_type: ActivityEventType::Started,
                    max_attempts: pending.max_attempts,
                    attempt_number,
                    created_at: Utc::now(),
                })
                .await;

                return Json(ServerEvent::PollActivityResponse(PollActivityResponse {
                    activity_run_id: pending.activity_run_id,
                    activity_id: pending.activity_id,
                    workflow_run_id: pending.workflow_run_id,
                    name,
                    input: pending.payload.clone(),
                    max_attempts: pending.max_attempts,
                    attempt_number,
                }));
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        },
        WorkerEvent::CompleteActivity {
            result,
            error,
            activity_id,
            activity_run_id,
            workflow_run_id,
            max_attempts,
            attempt_number,
        } => {
            if error.is_empty() {
                db.add_activity_event(ActivityEvent {
                    activity_id,
                    activity_run_id,
                    workflow_run_id,
                    event_type: ActivityEventType::Succeeeded,
                    payload: result,
                    created_at: Utc::now(),
                    max_attempts,
                    attempt_number,
                })
                .await;
            } else {
                db.add_activity_event(ActivityEvent {
                    activity_id,
                    activity_run_id,
                    workflow_run_id,
                    event_type: ActivityEventType::Failed,
                    payload: error,
                    created_at: Utc::now(),
                    max_attempts,
                    attempt_number,
                })
                .await;
            }
        }
        WorkerEvent::PollActivityCompletion { activity_run_id } => loop {
            if let Some(completed) = db.get_completed_activity(activity_run_id).await {
                match completed.event_type {
                    ActivityEventType::Succeeeded => {
                        return Json(ServerEvent::PollActivityCompletion(
                            PollActivityCompletion {
                                activity_run_id,
                                result: completed.payload,
                                error: "".to_string(),
                            },
                        ));
                    }
                    ActivityEventType::Failed => {
                        return Json(ServerEvent::PollActivityCompletion(
                            PollActivityCompletion {
                                activity_run_id,
                                error: completed.payload,
                                result: "".to_string(),
                            },
                        ));
                    }
                    _ => {}
                }
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        },
    };

    Json(ServerEvent::GeneralSuccess { success: true })
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: ServerState { db: Db::new() },
        }
    }

    pub async fn run(self) {
        let listener = tokio::net::TcpListener::bind("localhost:8080")
            .await
            .expect("Could not start server");

        println!("Starting service on 0.0.0.0:8080");
        let app = Router::new()
            .route("/worker_event", axum::routing::post(handle_worker_event))
            .route(
                "/rerun_workflow",
                axum::routing::post(handle_rerun_workflow),
            )
            .with_state(self.state.clone());

        axum::serve(listener, app).await.expect("Server crashed");
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
