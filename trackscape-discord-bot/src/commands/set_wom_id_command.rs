use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

/// Connects Trackscape to your Wise Old Man group.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "wom"
)]
pub async fn set_wom_id_command(
    ctx: Context<'_>,
    #[description = "Your Wise Old Man group id. Found on the side bar of your group page."]
    group_id: i64,
) -> Result<(), Error> {
    // Wise Old Man is called inline and can outlast Discord's three second reply window
    ctx.defer_ephemeral().await?;
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let wom_client = trackscape_discord_shared::wom::get_wom_client();
            match wom_client.group_client.get_group_details(group_id).await {
                Ok(details) => {
                    saved_guild.wom_id = Some(group_id);
                    db.guilds.update_guild(saved_guild).await;
                    format!(
                        "Connected to the Wise Old Man group \"{}\" ({} members).",
                        details.name, details.member_count
                    )
                }
                Err(_) => format!(
                    "Couldn't find a Wise Old Man group with the id {}. Double-check the group id on your group's page at wiseoldman.net.",
                    group_id
                ),
            }
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
