use crate::database::{BotMongoDb, MongoDb};

use redis::{Connection, RedisResult};
use std::env;
use wom_rs::WomClient;

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

pub fn get_wom_client() -> WomClient {
    let api_key = env::var("WOM_API_KEY").expect("WOM_API_KEY not set!");
    WomClient::new_with_key(api_key)
}
