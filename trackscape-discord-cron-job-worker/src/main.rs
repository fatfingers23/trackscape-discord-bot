use anyhow::Result;
use celery::beat::CronSchedule;
use dotenv::dotenv;
use env_logger::Env;
use trackscape_discord_shared::jobs::name_change_job::name_change;
use trackscape_discord_shared::jobs::wom_guild_sync_job::wom_guild_sync;

/// This is not really a worker but more of something to send jobs to the worker at certain times.
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let mut cron_job_worker = celery::beat!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            // "name_change" => {
            //     name_change,
            //     schedule = CronSchedule::from_string("30 4,16 * * *")?,
            //     args = (),
            // },
            "wom_guild_sync" => {
                wom_guild_sync,
                //Off set by at least 4 or 5 hours from name_change
                schedule = CronSchedule::from_string("0 0,12 * * *")?,
                args = (),
            }
        ],
        task_routes = [
            "*" => "cron_job_queue"
        ],
    ).await?;
    // name_change().await?;
    cron_job_worker.start().await?;
    Ok(())
}
