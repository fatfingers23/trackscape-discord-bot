use crate::commands::guild_id;
use crate::{Context, Error};

/// Stops leagues notifications, can reenable by setting the channel again.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn stop_leagues_notifications(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    let reply = match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(mut saved_guild) => {
                saved_guild.leagues_broadcast_channel = None;
                db.guilds.update_guild(saved_guild).await;
                "You have stopped receiving leagues notifications. You can reenable by setting the channel again."
            }
            None => {
                "Error finding your server as registered. Try kicking and re adding the bot please."
            }
        },
        Err(_) => "There was a technical error. Please try again later.",
    };
    ctx.say(reply).await?;
    Ok(())
}
