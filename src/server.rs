use std::sync::Arc;

use crate::activity_core::{Activity, ActivityEvent};
use crate::worker_events::WorkerEvent;
use crate::workflow_core::{Workflow, WorkflowEvent, WorkflowEventType};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct Db {
    workflows: Arc<RwLock<Vec<Workflow>>>,
    workflow_events: Arc<RwLock<Vec<WorkflowEvent>>>,

    activities: Arc<RwLock<Vec<Activity>>>,
    activity_events: Arc<RwLock<Vec<ActivityEvent>>>,
}

impl Db {
    fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(Vec::new())),
            workflow_events: Arc::new(RwLock::new(Vec::new())),

            activities: Arc::new(RwLock::new(Vec::new())),
            activity_events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub struct Server {
    db: Db,
}

#[axum::debug_handler]
async fn handle_worker_event(
    State(db): State<Db>,
    Json(input): Json<WorkerEvent>,
) -> impl IntoResponse {
    Json("Success")
}

impl Server {
    pub fn new() -> Self {
        Self { db: Db::new() }
    }

    async fn enqueu_workflow(&mut self, name: String, payload: String) {
        if let Some(workflow) = self
            .db
            .workflows
            .read()
            .await
            .iter()
            .find(|w| w.name == name)
        {
            self.db.workflow_events.write().await.push(WorkflowEvent {
                workflow_id: workflow.workflow_id,
                payload,
                workflow_event_type: WorkflowEventType::Pending,
                workflow_run_id: Uuid::new_v4(),
            })
        }
    }

    pub async fn run(self) {
        let listener = tokio::net::TcpListener::bind("localhost:8000")
            .await
            .expect("Could not start server");

        println!("Starting service on 0.0.0.0:8000");
        let app = Router::new()
            .route("/worker_event", axum::routing::post(handle_worker_event))
            .with_state(self.db.clone());

        axum::serve(listener, app).await.expect("Server crashed");
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
