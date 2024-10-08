use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{get_mongodb, get_redis_client, write_to_cache};
use crate::wom::{get_latest_name_change, get_wom_client};
use celery::prelude::*;
use redis::{Commands, RedisResult};

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
            //Checks to see if they had a name change
            let possible_saved_player = mongodb
                .clan_mates
                .find_by_current_name(player_name.clone())
                .await;

            match possible_saved_player {
                Ok(saved_player) => {
                    match saved_player {
                        None => {
                            //Player not found in db
                            let wom_client = get_wom_client();
                            let today = chrono::Utc::now().date_naive().format("%Y-%m-%d");
                            let redis_daily_calls_key = format!("api_calls:{}", today);
                            let _: () = redis_connection
                                .incr(redis_daily_calls_key.as_str(), 1)
                                .expect("failed to execute INCR for 'counter'");
                            let name_change_result =
                                get_latest_name_change(&wom_client, player_name.clone()).await;
                            if name_change_result.is_ok() {
                                let possible_name_change = name_change_result.unwrap();
                                match possible_name_change {
                                    None => {
                                        //No name change found, must be a new clan mate
                                        let _ = mongodb
                                            .clan_mates
                                            .create_new_clan_mate(
                                                guild_id,
                                                player_name.clone(),
                                                None,
                                            )
                                            .await;
                                    }
                                    Some(name_change) => {
                                        let check_by_old_name = mongodb
                                            .clan_mates
                                            .find_by_current_name(name_change.old_name.clone())
                                            .await;
                                        if check_by_old_name.is_err() {
                                            if check_by_old_name.unwrap().is_none() {
                                                let _ = mongodb
                                                    .clan_mates
                                                    .create_new_clan_mate(
                                                        guild_id,
                                                        player_name.clone(),
                                                        None,
                                                    )
                                                    .await;
                                            }
                                        } else {
                                            if check_by_old_name.unwrap().is_some() {
                                                let _ = mongodb
                                                    .clan_mates
                                                    .change_name(
                                                        guild_id,
                                                        name_change.old_name.clone(),
                                                        player_name.clone(),
                                                    )
                                                    .await;
                                            } else {
                                                let _ = mongodb
                                                    .clan_mates
                                                    .create_new_clan_mate(
                                                        guild_id,
                                                        player_name.clone(),
                                                        None,
                                                    )
                                                    .await;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(mut player) => {
                            if player.rank.is_none() {
                                player.rank = Some(rank.clone());

                                mongodb
                                    .clan_mates
                                    .update_clan_mate(player.clone())
                                    .await
                                    .unwrap();

                                write_to_cache(&mut redis_connection, redis_key, player.clone())
                                    .await;
                            }

                            if player.rank.clone().unwrap() == rank {
                                //No need to update rank is the same
                                return Ok(0);
                            }

                            player.rank = Some(rank.clone());
                            mongodb.clan_mates.update_clan_mate(player).await.unwrap();
                        }
                    }
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
