use crate::database::clan_mates::{ClanMateModel, ClanMates};
use crate::jobs::job_helpers::{get_mongodb, get_redis_client};
use celery::prelude::*;
use log::info;
use redis::{Commands, Connection, RedisResult};

#[celery::task]
pub async fn name_change() -> TaskResult<i32> {
    //Loop guilds Maybe fire off to other jobs?

    //Once you find a name has a newer name take the list of older ones and see if they need to be merged
    //Merge any of those results
    //Update the name
    info!("Starting name change job");
    Ok(4)
}
