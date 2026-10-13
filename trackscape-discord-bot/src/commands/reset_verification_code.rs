use crate::commands::guild_id;
use crate::{Context, Error};
use tracing::error;

/// Resets the verification code. MUST UPDATE THE NEW CODE IN THE PLUGIN.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn reset_verification_code(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let reset_code = ctx.data().db.guilds.reset_verification_code(guild_id).await;
    let reply = match reset_code {
        Ok(new_code) => format!("The new verification code is: {}", new_code),
        Err(e) => {
            error!("Failed to reset verification code: {:?}", e);
            "Failed to reset verification code.".to_string()
        }
    };
    ctx.say(reply).await?;
    Ok(())
}
