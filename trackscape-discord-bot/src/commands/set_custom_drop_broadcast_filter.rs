use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption,
};
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

pub fn register() -> CreateCommand {
    CreateCommand::new("custom_drop_broadcast_filter")
        .description("Sets a list of words/items to be filtered from the Broadcast channel.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "Filter list",
                "Enter a list of terms to filter drop broadcasts with, each separated by a comma e.g. rune, Torag, (g)",
            )
            .required(true),
        )
}

pub async fn run(
    command: &Vec<CommandDataOption>,
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;
    match saved_guild_query {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let filter = command.get(0).expect("Expected a list of values");

            if let CommandDataOptionValue::String(ref filter) = filter.value {
                let filter_list: Vec<String> = filter
                    .split(',')
                    .map(|s| s.trim().to_string()) 
                    .collect();
                saved_guild.custom_drop_broadcast_filter = Some(filter_list);
                db.guilds.update_guild(saved_guild).await;
                None
            } else {
                Some("Invalid filter.".to_string())
            }
        }
        Err(_) => {
            return Some(
                "No saved guild was found. Please try adding and removing the bot".to_string(),
            );
        }
    }
}
