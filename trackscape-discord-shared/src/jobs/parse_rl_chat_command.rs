use super::runelite_commands::pb_command::get_pb;
use celery::{error::TaskError, task::TaskResult};

#[celery::task(max_retries = 2)]
pub async fn parse_command(message: String, player: String, guild_id: u64) -> TaskResult<i32> {
    if message.to_lowercase().starts_with("!pb") {
        match get_pb(message, player, guild_id).await {
            Ok(_) => {}
            Err(e) => {
                println!("Error getting pb: {:?}", e);
                return Err(TaskError::ExpectedError(e.to_string()));
            }
        };
    }
    Ok(0)
}
