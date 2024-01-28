use crate::on_boarding_message::send_on_boarding;
use serenity::all::{CommandDataOption, CreateCommand};
use serenity::client::Context;
use serenity::model::id::ChannelId;

pub fn register() -> CreateCommand {
    CreateCommand::new("info").description("Displays info about the bot")
}

pub async fn run(
    _options: &[CommandDataOption],
    ctx: &Context,
    channel_id: ChannelId,
) -> Option<String> {
    send_on_boarding(channel_id, &ctx).await;
    None
}
