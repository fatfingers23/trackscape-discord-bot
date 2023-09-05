use serenity::builder;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::client::Context;
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

pub async fn run(options: &[CommandDataOption], ctx: &Context) -> Option<String> {

    let option = options
        .get(0)
        .expect("Expected Channel Id option")
        .resolved
        .as_ref()
        .expect("Expected Chanel Id object");

    if let CommandDataOptionValue::Channel(channel) = option {
        channel.id.send_message(&ctx.http, |m| {
            m.content(format!("This channel has been set as the clan chat channel."))
        }).await.unwrap();
    }
    None
}

