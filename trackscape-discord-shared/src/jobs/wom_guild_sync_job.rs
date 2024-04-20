use crate::jobs::job_helpers::get_mongodb;
use crate::jobs::wom_guild_sync_logic::sync_wom_by_guild;
use celery::task::TaskResult;
use log::info;

#[celery::task]
pub async fn wom_guild_sync() -> TaskResult<()> {
    //TODO need to look into the edge case of a name change may not be submitted yet to WOM,
    //So since it does not see that it makes a new clanmate. Probably just need to check
    // name changes for in db and combine after the loop and before remove?

    let mongodb = get_mongodb().await;

    let guilds = mongodb
        .guilds
        .list_clans()
        .await
        .expect("Failed to get all guilds");

    for guild in guilds {
        if guild.wom_id.is_none() {
            info!("No wom id found for guild: {:?}", guild.clan_name);
            continue;
        }
        sync_wom_by_guild(&guild, &mongodb).await;
    }

    Ok(())
}
