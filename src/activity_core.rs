use uuid::Uuid;

pub struct Activity {
    pub activity_id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub enum ActivityEventType {
    Started,
    Succeeeded,
    Failed,
}

#[derive(Debug)]
pub struct ActivityEvent {
    pub activity_id: Uuid,
    pub activity_run_id: Uuid,
    pub workflow_run_id: Uuid,
    pub activity_event_type: ActivityEventType,
    pub payload: String,
}

#[async_trait::async_trait]
pub trait AbstractActivityHandler: Send + Sync {
    async fn run(&self, input: String) -> Result<String, String>;
}
