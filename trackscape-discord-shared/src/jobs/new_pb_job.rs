use celery::prelude::*;

use crate::{
    database::clan_mates::ClanMates, jobs::job_helpers::get_mongodb,
    osrs_broadcast_extractor::osrs_broadcast_extractor::PersonalBestBroadcast,
};

#[celery::task]
pub async fn record_new_pb(pb: PersonalBestBroadcast, guild_id: u64) -> TaskResult<i32> {
    println!("Recording new PB: {:?}", pb);
    let db = get_mongodb().await;

    let activity_name = match pb.variant {
        Some(variant) => variant,
        None => pb.activity,
    };
    let clan_mate = db
        .clan_mates
        .find_or_create_clan_mate(guild_id, pb.player)
        .await;
    if clan_mate.is_err() {
        println!("Failed to find or create clan mate: {:?}", clan_mate.err());
        return Ok(1);
    }
    let activity = db.pb_activities.create_or_get_activity(activity_name).await;
    match activity {
        Ok(activity) => {
            let _ = db
                .pb_records
                .create_or_update_pb_record(
                    clan_mate.unwrap().id,
                    activity.id,
                    guild_id,
                    pb.time_in_seconds,
                )
                .await;
        }
        Err(e) => {
            println!("Failed to get activity: {:?}", e);
            return Ok(1);
        }
    };
    Ok(4)
}
