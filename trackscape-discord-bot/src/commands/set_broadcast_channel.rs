use crate::database::BotMongoDb;
use mongodb::error::Error;
use serenity::builder;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use tracing::info;
use trackscape_discord_shared::database::RegisteredGuild;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("set_broadcast_channel")
        .description("Sets a Channel to receive broadcasts. This will get embed messages for Pets, drops, pks, quests, etc")
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
        let saved_guild_query = db.get_by_guild_id(guild_id).await;

        return match saved_guild_query {
            Ok(possible_guild) => match possible_guild {
                Some(mut saved_guild) => {
                    saved_guild.broadcast_channel = Some(channel.id.0);
                    db.update_guild(saved_guild).await;
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
                            return Some(error.to_string());
                        }
                    }
                    //TODO: Send message to channel with verfication code and a picture of where to add it
                    Some("The channel has been set successfully".parse().unwrap())
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
