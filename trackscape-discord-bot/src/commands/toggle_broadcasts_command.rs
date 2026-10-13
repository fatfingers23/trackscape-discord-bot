use crate::commands::choices::ToggleBroadcast;
use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

/// Turns on or off a broadcast type to be sent.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "toggle"
)]
pub async fn toggle_broadcasts_command(
    ctx: Context<'_>,
    #[description = "Broadcast type to toggle on or off."] broadcast: ToggleBroadcast,
    #[description = "Turns on(true) or off(false) the broadcast type."] toggle: bool,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let broadcast_type = BroadcastType::from(broadcast);
            if toggle {
                saved_guild
                    .disallowed_broadcast_types
                    .retain(|b| b != &broadcast_type);
            } else {
                saved_guild
                    .disallowed_broadcast_types
                    .push(broadcast_type.clone());
            }

            db.guilds.update_guild(saved_guild).await;
            if toggle {
                format!(
                    "Successfully set to send {} broadcasts.",
                    broadcast_type.to_string()
                )
            } else {
                format!(
                    "Successfully set to NOT send {} broadcasts.",
                    broadcast_type.to_string()
                )
            }
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
