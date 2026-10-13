use crate::commands::guild_id;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tracing::{error, info};

/// Sets a Channel to receive broadcasts. This will get embed messages for Pets, drops, pks, quests, etc
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn set_broadcast_channel(
    ctx: Context<'_>,
    #[description = "The discord channel to get broadcast messages in."]
    #[channel_types("Text")]
    channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    info!("Guild Id: {} Channel Id: {}", guild_id, channel.id);
    if channel.kind != serenity::ChannelType::Text {
        error!("Please select a text channel.");
        ctx.say("Please select a text channel.").await?;
        return Ok(());
    }

    let saved_guild_query = ctx.data().db.guilds.get_by_guild_id(guild_id).await;

    let reply = match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(mut saved_guild) => {
                let saved_guild_code = saved_guild.verification_code.clone();
                saved_guild.broadcast_channel = Some(channel.id.get());
                ctx.data().db.guilds.update_guild(saved_guild).await;

                let send_message = channel
                    .id
                    .send_message(
                        ctx.http(),
                        serenity::CreateMessage::new()
                            .content("This channel has been set as the broadcast chat channel."),
                    )
                    .await;

                match send_message {
                    Ok(_) => format!(
                        "The channel has been set successfully. Please use the code: `{}` in the TrackScape Connection RuneLite Plugin to begin receiving messages in the selected channel.",
                        saved_guild_code
                    ),
                    Err(error) => {
                        info!("Error sending message: {}", error);
                        "Error sending a message to the selected channel. Please check that the bot has permission to access this channel. Broadcast messages will be sent once this is resolved.".to_string()
                    }
                }
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
