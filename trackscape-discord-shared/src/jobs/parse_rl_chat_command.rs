use celery::{error::TaskError, protocol::Message, task::TaskResult};

use super::{job_helpers::get_mongodb, runelite_commands::pb_command::get_pb};

#[celery::task]
pub async fn parse_command(message: String, player: String, guild_id: u64) -> TaskResult<i32> {
    let db = get_mongodb().await;

    //TODO make sure this is a return so it can fail if needed
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
