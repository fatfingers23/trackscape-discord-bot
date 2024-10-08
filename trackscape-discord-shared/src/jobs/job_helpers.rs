use crate::database::{BotMongoDb, MongoDb};
use redis::{Connection, RedisResult};
use serde::Serialize;
use std::env;

use redis::Commands;

pub async fn get_mongodb() -> BotMongoDb {
    let mongodb_url = env::var("MONGO_DB_URL").expect("MONGO_DB_URL not set!");
    BotMongoDb::new_db_instance(mongodb_url).await
}

pub fn get_redis_client() -> RedisResult<Connection> {
    let redis_url = env::var("REDIS_ADDR").expect("REDIS_ADDR not set!");
    redis::Client::open(redis_url)
        .expect("Could not connect to redis")
        .get_connection()
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
