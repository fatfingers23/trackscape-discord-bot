use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{clan_mate_cache_key, get_db, get_redis_connection, write_to_cache};
use crate::wom::{get_latest_name_change, get_wom_client};
use celery::prelude::*;
use chrono::{DateTime, Utc};
use log::error;
use redis::{Commands, RedisResult};

/// The most often we refresh a player's `last_seen_in_chat_at`. Chat is chatty,
/// so writing on every message would hammer the database.
const LAST_SEEN_WRITE_INTERVAL: chrono::Duration = chrono::Duration::hours(1);

/// Guests can talk in the clan channel but are not clan mates, so they get
/// no row, no rank update and no `last_seen_in_chat_at` touch.
pub fn is_guest_rank(rank: &str) -> bool {
    rank == "Guest"
}

/// Applies what a non-guest chat line tells us about a player: they are in
/// the clan (so a `left_at` marker is cleared) and they hold `rank`.
/// Returns true when the model changed and needs to be written back.
fn apply_chat_sighting(player: &mut ClanMateModel, rank: &str) -> bool {
    let mut changed = false;
    if player.left_at.is_some() {
        player.left_at = None;
        changed = true;
    }
    if player.rank.as_deref() != Some(rank) {
        player.rank = Some(rank.to_string());
        changed = true;
    }
    changed
}

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
    if is_guest_rank(&rank) {
        return Ok(0);
    }
    let mut redis_connection = match get_redis_connection() {
        Ok(redis_connection) => redis_connection,
        Err(e) => {
            error!("Failed to get redis client: {:?}", e);
            return Ok(0);
        }
    };
    let redis_key = clan_mate_cache_key(guild_id, &player_name);
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

        if apply_chat_sighting(&mut player, &rank) {
            if let Err(e) = db.clan_mates.update_clan_mate(player.clone()).await {
                error!("Failed to update clan mate from chat: {:?}", e);
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

            if apply_chat_sighting(&mut player, &rank) {
                if let Err(e) = db.clan_mates.update_clan_mate(player.clone()).await {
                    error!("Failed to update clan mate from chat: {:?}", e);
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
                // This branch always writes, so the "changed" result is not needed.
                apply_chat_sighting(&mut player, &rank);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guest_rank_is_not_a_clan_mate() {
        assert!(is_guest_rank("Guest"));
        assert!(!is_guest_rank("Owner"));
        assert!(!is_guest_rank("Not Ranked"));
        assert!(!is_guest_rank(""));
    }

    #[test]
    fn chat_line_restores_and_reranks() {
        let mut player = ClanMateModel::new(1, "Back".to_string(), None);
        player.left_at = Some(Utc::now());
        player.rank = Some("Sapphire".to_string());

        // Left member with the same rank: restored, needs a write.
        assert!(apply_chat_sighting(&mut player, "Sapphire"));
        assert_eq!(player.left_at, None);
        assert_eq!(player.rank.as_deref(), Some("Sapphire"));

        // Nothing changed: no write.
        assert!(!apply_chat_sighting(&mut player, "Sapphire"));

        // Rank changed: write.
        assert!(apply_chat_sighting(&mut player, "Ruby"));
        assert_eq!(player.rank.as_deref(), Some("Ruby"));

        // Left member who also changed rank: one write covers both.
        player.left_at = Some(Utc::now());
        assert!(apply_chat_sighting(&mut player, "Emerald"));
        assert_eq!(player.left_at, None);
        assert_eq!(player.rank.as_deref(), Some("Emerald"));
    }
}
