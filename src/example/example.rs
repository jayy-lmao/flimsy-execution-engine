use std::time::{Duration, Instant};

use crate::core;
use crate::server::Server;
use tokio::signal;

struct SumActivity;
#[async_trait::async_trait]
impl core::AbstractActivityHandler for SumActivity {
    async fn run(&self, input: String) -> Result<String, String> {
        let number = input.parse::<i32>().map_err(|_e| "Invalid string")?;
        Ok(format!("{}", number + 1))
    }
}
struct FailActivity;
#[async_trait::async_trait]
impl core::AbstractActivityHandler for FailActivity {
    async fn run(&self, _input: String) -> Result<String, String> {
        Err("Sadge".to_string())
    }
}

struct SumAndPrintWorkflow;
#[async_trait::async_trait]
impl core::AbstractWorkflowHandler for SumAndPrintWorkflow {
    async fn run(
        &self,
        mut context: core::WorkflowContext,
        input: String,
    ) -> Result<String, String> {
        let options = core::ActivityOptions {
            retry_policy: core::RetryOptions { max_attempts: 3 },
        };

        context.with_activity_options(options);

        let res = context.execute_activity(SumActivity, input.clone()).await?;

        let res_2 = context
            .execute_activity(FailActivity, "Fail input".to_string())
            .await?;

        Ok(format!("Processed {}, res_2 {}", res, res_2))
    }
}

pub async fn run() {
    println!("-------- Setting up -----");

    // New worker
    let server = Server::new();
    tokio::task::spawn(async move { server.run().await });

    let client = core::Client::new("http://localhost:8080");
    let mut worker = core::Worker::new(client);
    // Register workflow

    worker.register_activity(SumActivity).await;
    worker.register_activity(FailActivity).await;

    worker.register_workflow(SumAndPrintWorkflow).await;

    {
        let worker = worker.clone();
        tokio::task::spawn(async move { worker.run().await });
    }
    // {
    //     let worker = worker.clone();
    //     tokio::task::spawn(async move { worker.run().await });
    // }

    println!("-------- Running Test -----");
    tokio::time::sleep(Duration::from_millis(800)).await;

    let start = Instant::now();
    let res = worker
        .execute_workflow(SumAndPrintWorkflow, "3".to_string())
        .await;
    let execute_duration = start.elapsed();

    println!(
        "== Workflow res: {:?} in {}ms",
        res,
        execute_duration.as_millis()
    );

    // tokio::time::sleep(Duration::from_secs(2)).await;
    signal::ctrl_c()
        .await
        .expect("Failed to listen for SIGTERM");
}
