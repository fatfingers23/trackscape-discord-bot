use async_trait::async_trait;
use celery::error::CeleryError;
use celery::prelude::Task;
use celery::task::{AsyncResult, Signature};
use celery::Celery;
use std::sync::Arc;

pub mod add_job;
pub mod job_helpers;
pub mod name_change_job;
pub mod new_pb_job;
pub mod parse_rl_chat_command;
pub mod remove_clanmate_job;
mod runelite_commands;
pub mod update_create_clanmate_job;
pub mod wom_guild_sync_job;
pub mod wom_guild_sync_logic;

#[async_trait]
pub trait JobQueue {
    async fn send_task<T: Task>(&self, task_sig: Signature<T>) -> Result<AsyncResult, CeleryError>;
}

pub struct CeleryJobQueue {
    pub celery: Arc<Celery>,
}

pub async fn get_celery_caller() -> Arc<Celery> {
    celery::app!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
        ],
        // This just shows how we can route certain tasks to certain queues based
        // on glob matching.
        task_routes = [
            "*" => "celery",
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await.expect("Error creating celery job client")
}

#[async_trait]
impl JobQueue for CeleryJobQueue {
    async fn send_task<T: Task>(&self, task_sig: Signature<T>) -> Result<AsyncResult, CeleryError> {
        self.celery.send_task(task_sig).await
    }
}
