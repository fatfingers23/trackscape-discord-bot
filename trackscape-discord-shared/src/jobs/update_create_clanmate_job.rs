use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{get_db, get_redis_connection, write_to_cache};
use crate::wom::{get_latest_name_change, get_wom_client};
use celery::prelude::*;
use chrono::{DateTime, Utc};
use log::error;
use redis::{Commands, RedisResult};

/// The most often we refresh a player's `last_seen_in_chat_at`. Chat is chatty,
/// so writing on every message would hammer the database.
const LAST_SEEN_WRITE_INTERVAL: chrono::Duration = chrono::Duration::hours(1);

fn last_seen_is_stale(last_seen: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match last_seen {
        None => true,
        Some(last_seen) => now - last_seen > LAST_SEEN_WRITE_INTERVAL,
    }
}

///
/// Adds clan mates to the guild if they're not there already, and updates their rank if it's changed.
#[celery::task]
pub async fn update_create_clanmate(
    player_name: String,
    rank: String,
    guild_id: u64,
) -> TaskResult<i32> {
    let mut redis_connection = match get_redis_connection() {
        Ok(redis_connection) => redis_connection,
        Err(e) => {
            error!("Failed to get redis client: {:?}", e);
            return Ok(0);
        }
    };
    // Guild scoped so one clan's rank/last-seen state never leaks into another
    // clan that happens to have a member with the same name.
    let redis_key = format!("players:{}:{}", guild_id, player_name);
    let now = Utc::now();
    let db = get_db().await;

    let exists: RedisResult<bool> = redis_connection.exists(redis_key.clone());
    let does_key_exist = match exists {
        Ok(exists) => exists,
        Err(e) => {
            error!("Failed to check if key exists: {:?}", e);
            return Ok(0);
        }
    };

    if does_key_exist {
        let possible_player_cache: RedisResult<String> = redis_connection.get(redis_key.clone());
        let cached_player = match possible_player_cache {
            Ok(cached_player) => cached_player,
            Err(e) => {
                error!("Failed to get player cache: {:?}", e);
                return Ok(0);
            }
        };
        let mut player: ClanMateModel = match serde_json::from_str(&cached_player) {
            Ok(player) => player,
            Err(e) => {
                error!("Failed to deserialize cached player: {:?}", e);
                return Ok(0);
            }
        };

        if last_seen_is_stale(player.last_seen_in_chat_at, now) {
            if let Err(e) = db.clan_mates.touch_last_seen_in_chat(player.id).await {
                error!(
                    "Failed to update last seen in chat for player {}: {:?}",
                    player_name, e
                );
            }
            player.last_seen_in_chat_at = Some(now);
        }

        if player.rank.as_ref() != Some(&rank) {
            player.rank = Some(rank.clone());
            if let Err(e) = db.clan_mates.update_clan_mate(player.clone()).await {
                error!("Failed to update clan mate rank: {:?}", e);
                return Ok(0);
            }
        }

        write_to_cache(&mut redis_connection, redis_key, player).await;
        return Ok(0);
    }

    // Cache miss — look the player up in the database.
    let possible_saved_player = db
        .clan_mates
        .find_by_current_name(guild_id, player_name.clone())
        .await;

    match possible_saved_player {
        Ok(Some(mut player)) => {
            if last_seen_is_stale(player.last_seen_in_chat_at, now) {
                if let Err(e) = db.clan_mates.touch_last_seen_in_chat(player.id).await {
                    error!(
                        "Failed to update last seen in chat for player {}: {:?}",
                        player_name, e
                    );
                }
                player.last_seen_in_chat_at = Some(now);
            }

            if player.rank.as_ref() != Some(&rank) {
                player.rank = Some(rank.clone());
                if let Err(e) = db.clan_mates.update_clan_mate(player.clone()).await {
                    error!("Failed to update clan mate rank: {:?}", e);
                    return Ok(0);
                }
            }

            write_to_cache(&mut redis_connection, redis_key, player).await;
            return Ok(0);
        }
        Ok(None) => {
            // Not in the database: check WOM for a name change before deciding
            // whether this is a brand new clan mate or a rename.
            let wom_client = get_wom_client();
            let today = chrono::Utc::now().date_naive().format("%Y-%m-%d");
            let redis_daily_calls_key = format!("api_calls:{}", today);
            let incr_result: RedisResult<()> =
                redis_connection.incr(redis_daily_calls_key.as_str(), 1);
            if let Err(e) = incr_result {
                error!("Failed to incr the daily API call counter: {:?}", e);
            }

            let name_change = match get_latest_name_change(&wom_client, player_name.clone()).await {
                Ok(name_change) => name_change,
                Err(e) => {
                    error!(
                        "Failed to get latest name change for player {}: {:?}",
                        player_name, e
                    );
                    return Ok(0);
                }
            };

            let saved_player = match name_change {
                None => {
                    // No name change found, must be a new clan mate.
                    db.clan_mates
                        .create_new_clan_mate(guild_id, player_name.clone(), None)
                        .await
                        .map_err(|e| {
                            error!("Failed to create new clan mate {}: {:?}", player_name, e)
                        })
                        .ok()
                }
                Some(name_change) => {
                    let check_by_old_name = db
                        .clan_mates
                        .find_by_current_name(guild_id, name_change.old_name.clone())
                        .await;
                    match check_by_old_name {
                        Ok(Some(_)) => db
                            .clan_mates
                            .change_name(
                                guild_id,
                                name_change.old_name.clone(),
                                player_name.clone(),
                            )
                            .await
                            .map_err(|e| {
                                error!(
                                    "Failed to change name from {} to {}: {:?}",
                                    name_change.old_name, player_name, e
                                )
                            })
                            .ok(),
                        Ok(None) => db
                            .clan_mates
                            .create_new_clan_mate(guild_id, player_name.clone(), None)
                            .await
                            .map_err(|e| {
                                error!("Failed to create new clan mate {}: {:?}", player_name, e)
                            })
                            .ok(),
                        Err(e) => {
                            error!(
                                "Failed to look up player {} under old name {}: {:?}",
                                player_name, name_change.old_name, e
                            );
                            return Ok(0);
                        }
                    }
                }
            };

            if let Some(mut player) = saved_player {
                player.last_seen_in_chat_at = Some(now);
                if player.rank.is_none() {
                    player.rank = Some(rank.clone());
                }
                if let Err(e) = db.clan_mates.update_clan_mate(player.clone()).await {
                    error!("Failed to save clan mate {}: {:?}", player_name, e);
                    return Ok(0);
                }
                write_to_cache(&mut redis_connection, redis_key, player).await;
            }

            return Ok(0);
        }
        Err(e) => {
            error!(
                "Failed to look up saved player {} in guild {}: {:?}",
                player_name, guild_id, e
            );
            return Ok(0);
        }
    }
}
