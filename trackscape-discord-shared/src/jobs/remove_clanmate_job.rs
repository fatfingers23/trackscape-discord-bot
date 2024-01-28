use crate::database::clan_mates::ClanMates;
use crate::jobs::job_helpers::get_mongodb;
use celery::prelude::*;

///
/// Removes a clan mate
#[celery::task]
pub async fn remove_clanmate(player_name: String, guild_id: u64) -> TaskResult<i32> {
    let mongodb = get_mongodb().await;

    //TODO clean up other stuff?
    let result = mongodb
        .clan_mates
        .remove_clan_mate(guild_id, player_name)
        .await;
    match result {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to remove clan mate: {:?}", err);
        }
    }
    println!("update create clan mate job finished");
    Ok(4)
}
