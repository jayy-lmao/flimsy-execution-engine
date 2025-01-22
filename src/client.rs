use crate::worker_events::WorkerEvent;

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

    pub async fn register_workflow(&self, name: String) -> Result<(), String> {
        let event = WorkerEvent::RegisterWorkflow { name };
        let _ = self.client.post(&self.base_url).json(&event).send().await;
        Ok(())
    }
}
