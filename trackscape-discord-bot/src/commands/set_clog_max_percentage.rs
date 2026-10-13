use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

/// Set the max completion percentage for allowed broadcasts (see wiki collection log table)
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn set_clog_max_percentage(
    ctx: Context<'_>,
    #[description = "Collection log max completion %, max 100"] percentage: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            match percentage.parse::<f64>() {
                Ok(value) if (0.0..=100.0).contains(&value) => {
                    saved_guild.collection_log_max_percentage = Some(value);
                    db.guilds.update_guild(saved_guild).await;
                    "Successfully updated the Collection Log max percentage."
                }
                Ok(_) => "Percentage must be between 0 and 100.",
                Err(_) => "Invalid percentage value. Please provide a valid number.",
            }
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot",
    };
    ctx.say(reply).await?;
    Ok(())
}
