use crate::commands::guild_id;
use crate::{Context, Error};

/// Gets the verification code for the server to be used in the RuneLite plugin.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "get_verification_code"
)]
pub async fn get_verification_code(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let saved_guild_query = ctx.data().db.guilds.get_by_guild_id(guild_id).await;

    let reply = match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(saved_guild) => {
                let saved_guild_code = saved_guild.verification_code.clone();
                format!("The clan's verification code is: `{}`", saved_guild_code)
            }
            None => {
                "Error finding your server as registered. Try kicking and re adding the bot please."
                    .to_string()
            }
        },
        Err(_) => "There was a technical error. Please try again later.".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
