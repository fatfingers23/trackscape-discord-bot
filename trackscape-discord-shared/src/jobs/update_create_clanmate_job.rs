use crate::database::BotMongoDb;
use crate::jobs::job_helpers::{get_mongodb, get_redis_client};
use anyhow::Error;
use celery::prelude::*;
use redis::{Commands, JsonCommands, RedisResult, ToRedisArgs};
use redis_macros::{FromRedisValue, Json, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

#[derive(Debug, Clone, Serialize, Deserialize, FromRedisValue)]
struct CachedPlayer {
    pub player_name: String,
    pub rank: String,
    pub guild_id: u64,
    pub player_id: String,
}

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
        Err(_) => return Ok(0),
    };

    let cached_entry: RedisResult<String> =
        redis_connection.set_ex(redis_key.clone(), "test", 3600);

    let possible_player_cache: RedisResult<String> =
        redis_connection.get(format!("players:{}", player_name.clone()));
    let player_cache = match possible_player_cache {
        Ok(player_cache) => player_cache,
        Err(err) => {
            error!("Failed to get player cache: {:?}", err);
            return Ok(0);
        }
    };
    let mongodb = get_mongodb().await;
    println!("Hello from the job!");
    Ok(4)
}
