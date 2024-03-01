use anyhow::Result;
use celery::beat::CronSchedule;
use dotenv::dotenv;
use env_logger::Env;
use trackscape_discord_shared::jobs::name_change_job::name_change;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut cron_job_worker = celery::beat!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            
            //TODO do not think this is working
            "name_change" => {
                name_change,
                schedule = CronSchedule::from_string("*/1 * * * *")?,
                args = (),
            },
        ],
        task_routes = [
            "*" => "cron_job_queue"
        ],
    ).await?;
    // trackscape_discord_shared::jobs::name_change_job::name_change().await?;

    cron_job_worker.start().await?;
    Ok(())
}
