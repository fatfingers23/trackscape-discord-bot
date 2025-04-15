use crate::database::BotMongoDb;
use serenity::all::{CommandDataOption, CreateCommand};
use serenity::client::Context;
use serenity::model::prelude::Permissions;

pub fn register() -> CreateCommand {
    CreateCommand::new("get_custom_drop_broadcast_filter")
        .description("Gets the list of filters currently being used to filter the drop broadcasts.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
}

pub async fn run(
    _options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    return match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(saved_guild) => {
                let saved_guild_drop_filter = saved_guild.custom_drop_broadcast_filter.clone();
                Some(format!(
                    "The current drop broadcast filter list is: `{}`",
                    saved_guild_drop_filter.unwrap_or_default().join(", ")
                ))
            }
            None => Some(
                "Error finding your filter list. Please try again or set a new filter list"
                    .to_string(),
            ),
        },
        Err(_) => Some("There was a technical error. Please try again later.".to_string()),
    };
}
