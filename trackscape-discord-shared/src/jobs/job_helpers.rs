use crate::database::BotDb;
use log::error;
use redis::{Client, Connection, RedisResult};
use serde::Serialize;
use std::env;
use tokio::sync::OnceCell;

use redis::Commands;

/// A process-wide database facade for celery tasks. The old Mongo code built
/// a brand new client for every task invocation; with Postgres that would
/// open real connections each time, so the pool is built once and reused.
static DB: OnceCell<BotDb> = OnceCell::const_new();

pub async fn get_db() -> &'static BotDb {
    DB.get_or_init(|| async {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set!");
        BotDb::new_db_instance(db_url)
            .await
            .expect("Could not connect to the Postgres database")
    })
    .await
}

pub fn get_redis_connection() -> RedisResult<Connection> {
    let redis_url = env::var("REDIS_ADDR").expect("REDIS_ADDR not set!");
    redis::Client::open(redis_url)
        .expect("Could not connect to redis")
        .get_connection()
}

/// Like `get_redis_connection`, but never panics: a missing `REDIS_ADDR` or
/// an unreachable Redis is logged and returns `None`. For best-effort cache
/// work in code that also runs outside the workers (bot, wom-audit).
pub fn try_get_redis_connection() -> Option<Connection> {
    let redis_url = match env::var("REDIS_ADDR") {
        Ok(redis_url) => redis_url,
        Err(e) => {
            error!("REDIS_ADDR not set, skipping redis: {:?}", e);
            return None;
        }
    };
    match Client::open(redis_url).and_then(|client| client.get_connection()) {
        Ok(connection) => Some(connection),
        Err(e) => {
            error!("Failed to connect to redis: {:?}", e);
            None
        }
    }
}

/// Redis key under which the chat job caches a clan mate. Guild scoped so one
/// clan's state never leaks into another clan with a same-named member.
pub fn clan_mate_cache_key(guild_id: u64, player_name: &str) -> String {
    format!("players:{}:{}", guild_id, player_name)
}

/// Drops the chat job's cached copy of a clan mate, so the next chat line
/// reads the row from the database. Failures are logged, never fatal.
pub fn invalidate_clan_mate_cache(redis: &mut Connection, guild_id: u64, player_name: &str) {
    let key = clan_mate_cache_key(guild_id, player_name);
    let result: RedisResult<()> = redis.del(&key);
    if let Err(e) = result {
        error!("Failed to invalidate clan mate cache {}: {:?}", key, e);
    }
}

pub fn get_redis_client() -> Client {
    let redis_url = env::var("REDIS_ADDR").expect("REDIS_ADDR not set!");
    redis::Client::open(redis_url).expect("Could not connect to redis")
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clan_mate_cache_key_format() {
        assert_eq!(
            clan_mate_cache_key(42, "Some Player"),
            "players:42:Some Player"
        );
    }
}
