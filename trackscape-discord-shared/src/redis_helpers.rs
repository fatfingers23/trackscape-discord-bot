use log::error;
use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};

use redis::Commands;

pub async fn write_to_cache<T: Serialize>(
    redis_connection: &mut Connection,
    redis_key: String,
    data: T,
) {
    let _: RedisResult<String> = redis_connection.set_ex(
        redis_key.clone(),
        serde_json::to_string(&data).unwrap(),
        3600,
    );
}

pub async fn write_to_cache_with_seconds<T: Serialize>(
    redis_connection: &mut Connection,
    redis_key: &str,
    data: T,
    seconds: usize,
) {
    let result: RedisResult<String> =
        redis_connection.set_ex(redis_key, serde_json::to_string(&data).unwrap(), seconds);
    match result {
        Ok(_) => {}
        Err(err) => {
            error!("Error writing to redis cache: {}", err);
        }
    }
}

pub async fn fetch_redis_json_object<T: for<'a> Deserialize<'a>>(
    redis_connection: &mut Connection,
    redis_key: &str,
) -> Result<T, RedisFetchErrors> {
    let val = redis::cmd("GET")
        .arg(redis_key)
        .query::<String>(redis_connection)
        .map_err(|err| {
            error!("Error fetching from redis for the key {redis_key}: {}", err);
            RedisFetchErrors::FromDbError
        })?;

    let val: T = serde_json::from_str(&val).map_err(|err| {
        error!("Error parsing redis data: {}", err);
        RedisFetchErrors::ParseError
    })?;

    Ok(val)
}

pub async fn fetch_redis<T: redis::FromRedisValue>(
    redis_connection: &mut Connection,
    redis_key: &str,
) -> Result<T, RedisFetchErrors> {
    let val = redis::cmd("GET")
        .arg(redis_key)
        .query::<T>(redis_connection)
        .map_err(|_| RedisFetchErrors::FromDbError)?;

    Ok(val)
}

pub async fn redis_exists(redis_connection: &mut Connection, redis_key: &str) -> bool {
    redis_connection.exists(redis_key).unwrap_or(false)
}

#[derive(Debug)]
pub enum RedisFetchErrors {
    FromDbError,
    ParseError,
}
