use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

/// Hides your clan from the TrackScape website, or shows it again.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "hide_from_website"
)]
pub async fn hide_from_website_command(
    ctx: Context<'_>,
    #[description = "Hides(true) or shows(false) your clan on the TrackScape.app website."] hidden: bool,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            saved_guild.hidden_from_website = hidden;
            db.guilds.update_guild(saved_guild).await;
            if hidden {
                "Your clan is now hidden from the TrackScape website."
            } else {
                "Your clan is now visible on the TrackScape website."
            }
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot",
    };
    ctx.say(reply).await?;
    Ok(())
}
