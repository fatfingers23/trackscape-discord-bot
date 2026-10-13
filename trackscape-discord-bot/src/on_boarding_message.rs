use poise::serenity_prelude as serenity;
use serenity::{ChannelId, CreateEmbed, CreateMessage};
use tracing::error;

pub fn on_boarding_embed() -> CreateEmbed {
    CreateEmbed::default()
        .title("Welcome to TrackScape!")
        .url("https://github.com/fatfingers23/trackscape-discord-bot")
        .description("Thanks for adding TrackScape to your server! For this to work, make sure to install the TrackScape Connector plugin in RuneLite. This is how TrackScape gets the messages to send in discord.")
        .image("https://cdn.discordapp.com/attachments/961769668866088970/980601140603412510/220406_Trackscape_Logo-13.png")
        .field("Features", "* Sends in game clan chat to a discord channel of your choice
* Send messages from Discord to in game Clan Chat
* Sends embedded broadcasts of your clan's achievements. Including Pet Drops, High Value ", false)
        .field("Setup", "`/set_broadcast_channel` and `/set_clan_chat_channel` to make sure you have your channels set up to receive messages from the bot! When you set up either a Clan Chat or Broadcast channel a Code will be given. You will enter this in the settings of the RuneLite plugin.", false)
        .color(0x0000FF)
}

pub async fn send_on_boarding(channel: ChannelId, ctx: &serenity::Context) {
    let result = channel
        .send_message(
            &ctx.http,
            CreateMessage::default().embed(on_boarding_embed()),
        )
        .await;
    if let Err(e) = result {
        error!("Not able to send welcome message to system channel: {}", e);
    }
}
