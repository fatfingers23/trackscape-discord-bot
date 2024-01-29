use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{get_mongodb, get_redis_client, get_wom_client};
use celery::prelude::*;
use log::info;
use redis::{Commands, Connection, RedisResult};

// #[celery::task]
pub async fn name_change() -> TaskResult<i32> {
    //Loop guilds Maybe fire off to other jobs?
    let wom_client = get_wom_client();
    let player = wom_client
        .player_client
        .search("IFat Fingers".to_string(), None)
        .await;

    println!("Player: {:?}", player);
    println!("Starting name change job");
    Ok(4)
}
