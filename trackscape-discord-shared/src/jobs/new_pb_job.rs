use celery::prelude::*;

use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::PersonalBestBroadcast;

#[celery::task]
pub fn record_new_pb(pb: PersonalBestBroadcast, guild_id: u64) -> TaskResult<i32> {
    println!("Recording new PB: {:?}", pb);
    Ok(4)
}
