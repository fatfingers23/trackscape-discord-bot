use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use tracing::info;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("set_clan_chat_channel")
        .description("This channel will set the channel in game CC messages are sent to.")
        .create_option(|option| {
            option
                .name("channel")
                .description("The channel to set as the CC channel.")
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

        let server = db
            .get_by_guild_id(guild_id)
            .await
            .expect("Error getting server");
        match server {
            Some(mut server) => {
                server.clan_chat_channel = Some(channel.id.0);
                //TODO acutally need to set the channel in the db
                // db.update_guild(server).await;
                let result = channel
                    .id
                    .send_message(&ctx.http, |m| {
                        m.content(format!(
                            "This channel has been set as the clan chat channel."
                        ))
                    })
                    .await;

                match result {
                    Ok(_) => {}
                    Err(error) => {
                        info!("Error sending message: {}", error);
                        return Some(error.to_string());
                    }
                }
                return Some("The channel has been set succesfully".parse().unwrap());
            }
            None => {
                Some("Error finding your server as registered. Try kicking and re adding the bot please.".to_string());
            }
        }
    }
    None
}
