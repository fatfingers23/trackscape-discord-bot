use anyhow::Result;
use celery::beat::CronSchedule;
use dotenv::dotenv;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut cron_job_worker = celery::beat!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            "name_change" => {
                trackscape_discord_shared::jobs::name_change_job::name_change,
                schedule = CronSchedule::from_string("*/2 * * * *")?,
                args = (),
            },
        ],
        task_routes = [
            "*" => "cron_job_queue"
        ],
    ).await?;

    cron_job_worker.start().await?;

    Ok(())
}
