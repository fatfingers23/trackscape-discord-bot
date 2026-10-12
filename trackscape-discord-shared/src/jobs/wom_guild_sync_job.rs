use crate::jobs::job_helpers::get_db;
use crate::jobs::wom_guild_sync_logic::sync_wom_by_guild;
use celery::task::TaskResult;
use log::{error, info};

#[celery::task]
pub async fn wom_guild_sync() -> TaskResult<()> {
    let db = get_db().await;

    let guilds = match db.guilds.list_clans().await {
        Ok(guilds) => guilds,
        Err(e) => {
            error!("Failed to get all guilds for WOM sync: {:?}", e);
            return Ok(());
        }
    };

    for guild in guilds {
        if guild.wom_id.is_none() {
            info!("No wom id found for guild: {:?}", guild.clan_name);
            continue;
        }
        sync_wom_by_guild(&guild, &db).await;
    }

    Ok(())
}
