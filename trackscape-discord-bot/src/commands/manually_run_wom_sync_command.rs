use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::jobs::wom_guild_sync_logic::sync_wom_by_guild;

/// If WOM is connected it syncs between Trackscape and WOM.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "sync"
)]
pub async fn manually_run_wom_sync_command(ctx: Context<'_>) -> Result<(), Error> {
    // The sync talks to Wise Old Man and can outlast Discord's three second reply window
    ctx.defer_ephemeral().await?;
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            match saved_guild.wom_id {
                Some(_) => {
                    sync_wom_by_guild(&saved_guild, db).await;
                    "Sync run finished. If members look wrong, check the group id with /wom."
                }
                None => "Please set your WOM group's id first with /wom ",
            }
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot",
    };
    ctx.say(reply).await?;
    Ok(())
}
