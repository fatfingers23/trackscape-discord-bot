use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption, CreateMessage,
};
use serenity::client::Context;
use serenity::model::channel::ChannelType;

use serenity::model::prelude::Permissions;
use tracing::{error, info};

pub fn register() -> CreateCommand {
    let command = CreateCommand::new("set_clan_chat_channel");
    command
        .name("set_clan_chat_channel")
        .description("This channel will set the channel in game CC messages are sent to.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "channel",
                "The channel to set to receive ingame CC messages.",
            )
            .required(true),
        )
}

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let option = options.get(0).expect("Expected Channel Id option");

    if let CommandDataOptionValue::Channel(channel) = option.value {
        let possible_actual_channel = channel.to_channel(&ctx).await;
        if possible_actual_channel.is_err() {
            error!("Error getting channel: {:?}", possible_actual_channel.err());
            return Some("Error getting channel".to_string());
        }
        let guild_channel = possible_actual_channel
            .expect("Expected channel")
            .guild()
            .expect("Expected guild channel");

        if guild_channel.kind != ChannelType::Text {
            error!("Please select a text channel.");
            return Some("Please select a text channel.".to_string());
        }

        let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

        info!("Saved Guild: {:?}", saved_guild_query);
        return match saved_guild_query {
            Ok(possible_guild) => match possible_guild {
                Some(mut saved_guild) => {
                    let saved_guild_code = saved_guild.verification_code.clone();
                    saved_guild.clan_chat_channel = Some(channel.get());
                    db.guilds.update_guild(saved_guild).await;

                    let result = channel
                        .send_message(&ctx.http,
                            CreateMessage::new().content("This channel has been set as the clan chat channel.".to_string()))
                        .await;

                    match result {
                        Ok(_) => {}
                        Err(error) => {
                            info!("Error sending message: {}", error);
                            return Some("Error sending a message to the selected channel. Please check that the bot has permission to access this channel. Clan Chat messages will be sent once this is resolved.".to_string())                        }
                    }
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
