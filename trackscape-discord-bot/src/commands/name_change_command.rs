use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::clan_mates::ClanMates;

/// Changes a clanmates name.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "name_change"
)]
pub async fn name_change_command(
    ctx: Context<'_>,
    #[description = "The old name of the clanmate. Case sensitive."] old: String,
    #[description = "The new name of the clanmate. Case sensitive."] new: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let result = ctx
        .data()
        .db
        .clan_mates
        .change_name(
            guild_id,
            old.replace(" ", "\u{a0}"),
            new.replace(" ", "\u{a0}"),
        )
        .await;
    let reply = match result {
        Ok(_) => format!("Successfully changed {} to {}.", old, new),
        Err(_) => format!("Error changing {} to {}.", old, new),
    };
    ctx.say(reply).await?;
    Ok(())
}
