use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{get_mongodb, get_redis_client};
use celery::prelude::*;
use redis::{Commands, Connection, RedisResult};

///
/// Adds clan mates to the guild if they're not there already, and updates their rank if it's changed.
#[celery::task]
pub async fn update_create_clanmate(
    player_name: String,
    rank: String,
    guild_id: u64,
) -> TaskResult<i32> {
    let mut redis_connection = get_redis_client().expect("Failed to get redis client.");
    let redis_key = format!("players:{}", player_name.clone());
    let exists: RedisResult<bool> = redis_connection.exists(redis_key.clone());

    let does_key_exist = match exists {
        Ok(exists) => exists,
        Err(err) => {
            println!("Failed to check if key exists: {:?}", err);
            return Ok(0);
        }
    };
    let mongodb = get_mongodb().await;

    match does_key_exist {
        true => {
            let possible_player_cache: RedisResult<String> =
                redis_connection.get(redis_key.clone());
            match possible_player_cache {
                Ok(cached_player) => {
                    let mut serialized_player: ClanMateModel =
                        serde_json::from_str(&cached_player).unwrap();

                    if serialized_player.rank.is_none() {
                        serialized_player.rank = Some(rank.clone());
                    }
                    if serialized_player.rank.clone().unwrap() == rank {
                        //No need to update rank is the same
                        return Ok(0);
                    }

                    serialized_player.rank = Some(rank.clone());
                    mongodb
                        .clan_mates
                        .update_clan_mate(serialized_player.clone())
                        .await
                        .unwrap();
                    write_to_cache(&mut redis_connection, redis_key, serialized_player).await;
                }
                Err(err) => {
                    println!("Failed to get player cache: {:?}", err);
                }
            }
        }
        false => {
            let possible_saved_player = mongodb
                .clan_mates
                .find_or_create_clan_mate(guild_id, player_name.clone())
                .await;

            match possible_saved_player {
                Ok(mut saved_player) => {
                    if saved_player.rank.is_none() {
                        saved_player.rank = Some(rank.clone());

                        mongodb
                            .clan_mates
                            .update_clan_mate(saved_player.clone())
                            .await
                            .unwrap();

                        write_to_cache(&mut redis_connection, redis_key, saved_player.clone())
                            .await;
                    }

                    if saved_player.rank.clone().unwrap() == rank {
                        //No need to update rank is the same
                        return Ok(0);
                    }

                    saved_player.rank = Some(rank.clone());
                    mongodb
                        .clan_mates
                        .update_clan_mate(saved_player)
                        .await
                        .unwrap();
                }
                Err(err) => {
                    println!("Failed to save or create player: {:?}", err);
                }
            }
        }
    }

    println!("update create clan mate job finished");
    Ok(4)
}

async fn write_to_cache(
    redis_connection: &mut Connection,
    redis_key: String,
    updated_player: ClanMateModel,
) {
    let _: RedisResult<String> = redis_connection.set_ex(
        redis_key.clone(),
        serde_json::to_string(&updated_player).unwrap(),
        3600,
    );
}
