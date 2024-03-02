use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let instant_job_app = celery::app!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            trackscape_discord_shared::jobs::add_job::run,
            trackscape_discord_shared::jobs::update_create_clanmate_job::update_create_clanmate,
            trackscape_discord_shared::jobs::remove_clanmate_job::remove_clanmate,
            trackscape_discord_shared::jobs::name_change_job::name_change,
        ],
        // This just shows how we can route certain tasks to certain queues based
        // on glob matching.
        task_routes = [
            "name_change" => "cron_job_queue",
            "*" => "celery",
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await?;

    instant_job_app.display_pretty().await;
    instant_job_app
        .consume_from(&["celery", "cron_job_queue"])
        .await?;

    instant_job_app.close().await?;

    Ok(())
}
