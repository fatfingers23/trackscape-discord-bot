use anyhow::Result;
use dotenv::dotenv;
use env_logger::Env;
use trackscape_discord_shared::jobs::{
    add_job, name_change_job::name_change, new_pb_job::record_new_pb,
    parse_rl_chat_command::parse_command, remove_clanmate_job::remove_clanmate,
    update_create_clanmate_job::update_create_clanmate, wom_guild_sync_job::wom_guild_sync,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let instant_job_app = celery::app!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            add_job::run,
            update_create_clanmate,
            remove_clanmate,
            name_change,
            wom_guild_sync,
            record_new_pb,
            parse_command,
        ],
        // This just shows how we can route certain tasks to certain queues based
        // on glob matching.
        task_routes = [
            "name_change" => "cron_job_queue",
            "wom_guild_sync" => "cron_job_queue",
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
