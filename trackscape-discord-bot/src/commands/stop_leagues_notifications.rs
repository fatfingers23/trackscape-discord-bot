use crate::on_boarding_message::send_on_boarding;
use serenity::all::{CommandDataOption, CreateCommand};
use serenity::client::Context;
use serenity::model::id::ChannelId;
use trackscape_discord_shared::database::BotMongoDb;

pub fn register() -> CreateCommand {
    CreateCommand::new("stop_leagues_notifications")
        .description("Stops leagues notifications, can reenable by setting the channel again.")
}

pub async fn run(
    _options: &[CommandDataOption],
    ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    return match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(mut saved_guild) => {
                saved_guild.leagues_broadcast_channel = None;
                db.guilds.update_guild(saved_guild).await;
                Some("You have stopped receiving leagues notifications. You can reenable by setting the channel again.".to_string())
            }
            None => {
                Some("Error finding your server as registered. Try kicking and re adding the bot please.".to_string())
            }
        },
        Err(_) => {
            Some("There was a technical error. Please try again later.".to_string())
        }
    };
    None
}
