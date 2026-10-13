use crate::commands::guild_id;
use crate::{Context, Error};
use tracing::error;
use trackscape_discord_shared::database::clan_mates::ClanMates;

/// Removes a clanmate from the clan in TrackScape.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "expel"
)]
pub async fn expel_clanmate_command(
    ctx: Context<'_>,
    #[description = "Player to mark as having left the clan."] rsn: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    // No Redis here: the chat job's cached copy may delay a chat-driven restore by up to an hour.
    let result = ctx
        .data()
        .db
        .clan_mates
        .soft_remove_clan_mate(guild_id, rsn.clone())
        .await;
    let reply = match result {
        Ok(_) => format!("Marked {} as having left the clan.", rsn),
        Err(err) => {
            error!("Error marking {} as having left the clan: {:?}", rsn, err);
            if err.to_string().contains("Failed to find clan mate") {
                format!("Could not find {} in this clan's TrackScape members.", rsn)
            } else {
                format!("Error marking {} as having left the clan.", rsn)
            }
        }
    };
    ctx.say(reply).await?;
    Ok(())
}
