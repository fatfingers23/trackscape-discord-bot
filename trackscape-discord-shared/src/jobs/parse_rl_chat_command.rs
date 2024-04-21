use celery::{protocol::Message, task::TaskResult};

use super::{job_helpers::get_mongodb, runelite_commands::pb_command::get_pb};

#[celery::task]
pub async fn parse_command(message: String, player: String, guild_id: u64) -> TaskResult<i32> {
    let db = get_mongodb().await;

    if message.to_lowercase().starts_with("!pb") {
        get_pb(message, player, guild_id).await;
    }
    Ok(4)
}
