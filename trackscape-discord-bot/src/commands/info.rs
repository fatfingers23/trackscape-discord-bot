use crate::on_boarding_message::send_on_boarding;
use serenity::builder;
use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::prelude::application_command::CommandDataOption;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("info")
        .description("Displays info about the bot")
}

pub async fn run(
    _options: &[CommandDataOption],
    ctx: &Context,
    channel_id: ChannelId,
) -> Option<String> {
    send_on_boarding(channel_id, &ctx).await;
    None
}
