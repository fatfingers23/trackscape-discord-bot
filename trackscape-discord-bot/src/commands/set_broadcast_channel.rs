use crate::database::BotMongoDb;

use serenity::builder;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use tracing::info;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("set_broadcast_channel")
        .description("Sets a Channel to receive broadcasts. This will get embed messages for Pets, drops, pks, quests, etc")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("channel")
                .description("The discord channel to get broadcast messages in.")
                .kind(CommandOptionType::Channel)
                .required(true)
        })
}

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let option = options
        .get(0)
        .expect("Expected Channel Id option")
        .resolved
        .as_ref()
        .expect("Expected Chanel Id object");
    if let CommandDataOptionValue::Channel(channel) = option {
        if channel.kind != ChannelType::Text {
            return Some("Please select a text channel.".to_string());
        }
        info!("Channel: {:?}", channel);
        let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

        return match saved_guild_query {
            Ok(possible_guild) => match possible_guild {
                Some(mut saved_guild) => {
                    let saved_guild_code = saved_guild.verification_code.clone();
                    saved_guild.broadcast_channel = Some(channel.id.0);
                    db.guilds.update_guild(saved_guild).await;
                    let send_message = channel
                        .id
                        .send_message(&ctx.http, |m| {
                            m.content(format!(
                                "This channel has been set as the broadcast chat channel."
                            ))
                        })
                        .await;

                    match send_message {
                        Ok(_) => {}
                        Err(error) => {
                            info!("Error sending message: {}", error);
                            return Some("Error sending a message to the selected channel. Please check that the bot has permission to access this channel. Broadcast messages will be sent once this is resolved.".to_string())
                        }
                    }
                    //TODO: Send message to channel with verfication code and a picture of where to add it
                    Some(format!("The channel has been set successfully. Please use the code: `{}` in the TrackScape Connection RuneLite Plugin to begin receiving messages in the selected channel.", saved_guild_code))
                }
                None => {
                    Some("Error finding your server as registered. Try kicking and re adding the bot please.".to_string())
                }
            },
            Err(_) => {
                Some("There was a technical error. Please try again later.".to_string())
            }
        };
    }
    info!("Error getting channel");
    Some("Error getting channel".to_string())
}
