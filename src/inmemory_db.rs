use std::sync::Arc;

use crate::core::activity::{
    Activity, ActivityEvent, ActivityEventType, ActivityId, ActivityName, ActivityRunId,
};
use crate::core::workflow::{
    Workflow, WorkflowEvent, WorkflowEventType, WorkflowId, WorkflowName, WorkflowRunId,
};
use dashmap::DashMap;

#[derive(Clone)]
pub struct Db {
    pub workflows: Arc<DashMap<WorkflowName, Workflow>>,
    pub workflow_runs: Arc<DashMap<WorkflowId, Vec<WorkflowRunId>>>,
    pub workflow_events: Arc<DashMap<WorkflowRunId, Vec<WorkflowEvent>>>,

    pub activities: Arc<DashMap<ActivityName, Activity>>,
    pub activity_runs: Arc<DashMap<ActivityId, Vec<ActivityRunId>>>,
    pub activity_events: Arc<DashMap<ActivityRunId, Vec<ActivityEvent>>>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(DashMap::new()),
            workflow_runs: Arc::new(DashMap::new()),
            workflow_events: Arc::new(DashMap::new()),

            activities: Arc::new(DashMap::new()),
            activity_runs: Arc::new(DashMap::new()),
            activity_events: Arc::new(DashMap::new()),
        }
    }

    pub async fn workflow_exists(&self, name: &WorkflowName) -> bool {
        self.workflows.get(name).is_some()
    }

    pub async fn get_workflow_by_name(&self, name: &WorkflowName) -> Option<Workflow> {
        let workflow = self.workflows.get(name)?;
        Some(workflow.clone())
    }

    pub async fn activity_exists(&self, name: &ActivityName) -> bool {
        self.activities.get(name).is_some()
    }

    pub async fn get_activity_by_name(&self, name: &ActivityName) -> Option<Activity> {
        let activity = self.activities.get(name)?;
        Some(activity.clone())
    }

    pub async fn add_activity(&self, activity: Activity) {
        self.activities.insert(activity.name.clone(), activity);
    }

    pub async fn add_workflow(&self, workflow: Workflow) {
        self.workflows.insert(workflow.name.clone(), workflow);
    }

    pub async fn add_activity_event(&self, event: ActivityEvent) {
        self.activity_runs
            .entry(event.activity_id)
            .and_modify(|runs| {
                if !runs.contains(&event.activity_run_id) {
                    runs.push(event.activity_run_id);
                }
            })
            .or_insert(vec![event.activity_run_id]);

        self.activity_events
            .entry(event.activity_run_id)
            .and_modify(|events| events.push(event.clone()))
            .or_insert(vec![event]);
    }

    pub async fn add_workflow_event(&self, event: WorkflowEvent) {
        self.workflow_runs
            .entry(event.workflow_id)
            .and_modify(|runs| {
                if !runs.contains(&event.run_id) {
                    runs.push(event.run_id);
                }
            })
            .or_insert(vec![event.run_id]);

        self.workflow_events
            .entry(event.run_id)
            .and_modify(|events| events.push(event.clone()))
            .or_insert(vec![event]);
    }

    pub async fn get_first_pending_workflow(&self, name: WorkflowName) -> Option<WorkflowEvent> {
        let workflow = self.workflows.get(&name)?;
        let runs = self.workflow_runs.get(&workflow.id)?.clone();

        for run in runs {
            let last_event = self.get_last_workflow_run_event(run).await?;

            if last_event.event_type == WorkflowEventType::Pending {
                return Some(last_event);
            }
        }
        None
    }

    pub async fn get_first_pending_activity(&self, name: ActivityName) -> Option<ActivityEvent> {
        let activity = self.activities.get(&name)?;
        let runs = self.activity_runs.get(&activity.id)?.clone();

        for run in runs {
            let last_event = self.get_last_activity_run_event(run).await?;

            if last_event.event_type == ActivityEventType::Pending {
                return Some(last_event);
            }
        }
        None
    }

    pub async fn get_success_activity_event_for_run(
        &self,
        past_workflow_run_id: WorkflowRunId,
        activity_id: ActivityId,
        input: &str,
    ) -> Option<ActivityEvent> {
        let runs = self.activity_runs.get(&activity_id)?;

        for activity_run in runs.iter() {
            let last_event = self.get_last_activity_run_event(*activity_run).await?;

            if last_event.workflow_run_id == past_workflow_run_id && last_event.payload == input {
                return Some(last_event);
            }
        }
        None
    }

    pub async fn get_completed_activity(
        &self,
        activity_run_id: ActivityRunId,
    ) -> Option<ActivityEvent> {
        let activity_events = self.activity_events.get(&activity_run_id)?;
        let success_event = activity_events
            .iter()
            .find(|we| we.event_type == ActivityEventType::Succeeeded);

        if success_event.is_some() {
            return success_event.cloned();
        }

        let failed_event = activity_events
            .iter()
            .find(|we| we.event_type == ActivityEventType::Failed);

        if failed_event.is_some() {
            return failed_event.cloned();
        }

        None
    }

    pub async fn get_completed_workflow(
        &self,
        workflow_run_id: WorkflowRunId,
    ) -> Option<WorkflowEvent> {
        let workflow_events = self.workflow_events.get(&workflow_run_id)?;

        let success_event = workflow_events
            .iter()
            .find(|we| we.event_type == WorkflowEventType::Succeeeded);

        if success_event.is_some() {
            return success_event.cloned();
        }

        let failed_event = workflow_events
            .iter()
            .find(|we| we.event_type == WorkflowEventType::Failed);

        if failed_event.is_some() {
            return failed_event.cloned();
        }
        None
    }

    pub async fn get_last_activity_run_event(
        &self,
        activity_run_id: ActivityRunId,
    ) -> Option<ActivityEvent> {
        let mut activity_events = self.activity_events.get(&activity_run_id)?.clone();
        activity_events.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        activity_events.last().cloned()
    }

    pub async fn get_last_workflow_run_event(
        &self,
        workflow_run_id: WorkflowRunId,
    ) -> Option<WorkflowEvent> {
        let mut workflow_events = self.workflow_events.get(&workflow_run_id)?.clone();
        workflow_events.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        workflow_events.last().cloned()
    }
}

impl Default for Db {
    fn default() -> Self {
        Self::new()
    }
}
