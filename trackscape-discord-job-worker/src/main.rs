use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let my_app = celery::app!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            trackscape_discord_shared::jobs::add_job::run,
            trackscape_discord_shared::jobs::update_create_clanmate_job::update_create_clanmate,
            trackscape_discord_shared::jobs::remove_clanmate_job::remove_clanmate,
        ],
        // This just shows how we can route certain tasks to certain queues based
        // on glob matching.
        task_routes = [
            "*" => "celery",
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await?;

    my_app.display_pretty().await;
    my_app.consume_from(&["celery"]).await?;

    my_app.close().await?;

    Ok(())
}
