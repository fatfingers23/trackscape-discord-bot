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
    let _: RedisResult<String> =
        redis_connection.set_ex(redis_key, serde_json::to_string(&data).unwrap(), seconds);
}

pub async fn fetch_redis_json_object<T: for<'a> Deserialize<'a>>(
    redis_connection: &mut Connection,
    redis_key: &str,
) -> T {
    let val = redis::cmd("GET")
        .arg(redis_key)
        .query::<String>(redis_connection)
        .unwrap();

    let val: T = serde_json::from_str(&val).unwrap();

    val
}
