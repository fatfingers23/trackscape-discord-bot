use crate::commands::choices::DiaryTierChoice;
use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

/// Sets min diary tier to trigger a broadcast. Anything below will not send
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "diaries"
)]
pub async fn set_diary_min_command(
    ctx: Context<'_>,
    #[description = "Min diary tier difficulty to send a broadcast."] tier: DiaryTierChoice,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            saved_guild.min_diary_tier = Some(tier.into());
            db.guilds.update_guild(saved_guild).await;
            "Successfully updated min diary to broadcast."
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot",
    };
    ctx.say(reply).await?;
    Ok(())
}
