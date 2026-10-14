use crate::commands::choices::ClearableChannel;
use crate::commands::guild_id;
use crate::{Context, Error};

/// Clears a set channel so the bot stops sending to it. Can reenable by setting the channel again.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn clear_channel(
    ctx: Context<'_>,
    #[description = "The channel setting to clear."] channel: ClearableChannel,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    let reply = match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(mut saved_guild) => {
                let saved_channel = match channel {
                    ClearableChannel::Broadcast => &mut saved_guild.broadcast_channel,
                    ClearableChannel::ClanChat => &mut saved_guild.clan_chat_channel,
                    ClearableChannel::Leagues => &mut saved_guild.leagues_broadcast_channel,
                };
                match saved_channel.take() {
                    Some(_) => {
                        db.guilds.update_guild(saved_guild).await;
                        cleared_reply(channel)
                    }
                    None => "That channel is not set, so there was nothing to clear.",
                }
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

fn cleared_reply(channel: ClearableChannel) -> &'static str {
    match channel {
        ClearableChannel::Broadcast => {
            "The broadcast channel has been cleared. You can reenable by using `/set_broadcast_channel`."
        }
        ClearableChannel::ClanChat => {
            "The clan chat channel has been cleared. Discord messages will also no longer be sent to the in game clan chat. You can reenable by using `/set_clan_chat_channel`."
        }
        ClearableChannel::Leagues => {
            "The leagues broadcast channel has been cleared. You can reenable by using `/set_leagues_channel`."
        }
    }
}
