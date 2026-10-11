use crate::database::BotDb;
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
