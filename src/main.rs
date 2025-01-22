use activity_core::AbstractActivityHandler;
use client::Client;
use server::Server;
use worker_core::Worker;
use workflow_core::{AbstractWorkflowHandler, WorkflowContext};

pub mod activity_core;
/// This event-registry is based on Type-Driven API Design in Rust.
/// see: https://willcrichton.net/rust-api-type-patterns/registries.html
/// Only major change is the support of dependency injection via a single Arc.
pub mod worker_core;
pub mod workflow_core;

pub mod client;
pub mod server;
pub mod worker_events;

struct SumActivity;
#[async_trait::async_trait]
impl AbstractActivityHandler for SumActivity {
    async fn run(&self, input: String) -> Result<String, String> {
        let number = input.parse::<i32>().map_err(|_e| "Invalid string")?;
        Ok(format!("{}", number + 1))
    }
}

struct SumAndPrintWorkflow;
#[async_trait::async_trait]
impl AbstractWorkflowHandler for SumAndPrintWorkflow {
    async fn run(&self, context: WorkflowContext, input: String) -> Result<String, String> {
        let res = context
            .execute_activity("SumActivity", input.clone())
            .await?;
        Ok(format!("INPUT: {}, OUTPUT: {}", input, res))
    }
}

#[tokio::main]
async fn main() {
    // New worker
    let server = Server::new();
    tokio::task::spawn(async move { server.run().await });

    let client = Client::new("http://localhost:8080");
    let mut worker = Worker::new(client);
    // Register workflow

    worker.register_activity("SumActivity", SumActivity).await;

    worker
        .register_workflow("SumAndPrintWorkflow", SumAndPrintWorkflow)
        .await;

    let res = worker
        .execute_workflow("SumAndPrintWorkflow", "3".to_string())
        .await;

    // println!("Workflow events {:#?}", worker.workflow_events);
    // println!("Activity events {:#?}", worker.activity_events);
    println!("== Workflow res: {:?}", res);
}
