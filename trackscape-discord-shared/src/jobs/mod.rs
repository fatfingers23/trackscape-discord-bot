use async_trait::async_trait;
use celery::error::CeleryError;
use celery::prelude::Task;
use celery::task::{AsyncResult, Signature};
use celery::{error, Celery};
use std::sync::Arc;

pub mod add_job;
mod job_helpers;
pub mod update_create_clanmate_job;

#[async_trait]
pub trait JobQueue {
    async fn send_task<T: Task>(&self, task_sig: Signature<T>) -> Result<AsyncResult, CeleryError>;
}

pub struct CeleryJobQueue {
    pub celery: Arc<Celery>,
}

#[async_trait]
impl JobQueue for CeleryJobQueue {
    async fn send_task<T: Task>(&self, task_sig: Signature<T>) -> Result<AsyncResult, CeleryError> {
        self.celery.send_task(task_sig).await
    }
}
