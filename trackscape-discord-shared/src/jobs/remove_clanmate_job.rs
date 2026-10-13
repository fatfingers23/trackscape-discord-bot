use crate::database::clan_mates::ClanMates;
use crate::jobs::job_helpers::{get_db, invalidate_clan_mate_cache, try_get_redis_connection};
use celery::prelude::*;
use log::error;

/// Marks a clan mate as having left after a "has left the clan" or
/// "has expelled … from the clan" broadcast. Their history is kept.
#[celery::task]
pub async fn remove_clanmate(player_name: String, guild_id: u64) -> TaskResult<i32> {
    let db = get_db().await;
    match db
        .clan_mates
        .soft_remove_clan_mate(guild_id, player_name.clone())
        .await
    {
        Ok(removed) => {
            // The chat job caches the model (with `left_at = None`); drop it so a
            // rejoin's next chat line reads the row and restores them. Both
            // spellings: the broadcast's (how the chat job keyed it) and the
            // stored one, in case the casing differs.
            if let Some(mut redis) = try_get_redis_connection() {
                invalidate_clan_mate_cache(&mut redis, guild_id, &player_name);
                invalidate_clan_mate_cache(&mut redis, guild_id, &removed.player_name);
            }
        }
        Err(err) => {
            error!(
                "Failed to mark {} as left in guild {}: {:?}",
                player_name, guild_id, err
            );
        }
    }
    Ok(0)
}
