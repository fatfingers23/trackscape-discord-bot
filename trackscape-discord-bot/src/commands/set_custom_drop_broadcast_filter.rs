use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption,
};
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

pub fn register() -> CreateCommand {
    CreateCommand::new("set_custom_drop_broadcast_filter")
        .description("Sets a list of filter words to use with the selected broadcast type.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "broadcast",
                "Broadcast type to set a filter list for.",
            )
            .add_string_choice(
                BroadcastType::ItemDrop.to_string(),
                BroadcastType::ItemDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::RaidDrop.to_string(),
                BroadcastType::RaidDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::PetDrop.to_string(),
                BroadcastType::PetDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CollectionLog.to_string(),
                BroadcastType::CollectionLog.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Quest.to_string(),
                BroadcastType::Quest.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Diary.to_string(),
                BroadcastType::Diary.to_slug(),
            )
            .add_string_choice(
                BroadcastType::PersonalBest.to_string(),
                BroadcastType::PersonalBest.to_slug(),
            )
            .add_string_choice(
                BroadcastType::ClueItem.to_string(),
                BroadcastType::ClueItem.to_slug(),
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "filter",
                "Enter a comma-separated list to filter broadcasts with",
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
            let broadcast_type = command.get(0).expect("Expected a broadcast type option");

            let filter = command
                .get(1)
                .expect("Expected a comma-separated list of values");

            return if let CommandDataOptionValue::String(broadcast_type) =
                broadcast_type.clone().value
            {
                if let CommandDataOptionValue::String(filter) = filter.clone().value {
                    let broadcast_type =
                        BroadcastType::from_string(broadcast_type.replace("_", " "));

                    let filter_list: Vec<String> =
                        filter.split(',').map(|s| s.trim().to_string()).collect();
                    if let Some(ref mut filter_map) = saved_guild.custom_drop_broadcast_filter {
                        filter_map.insert(broadcast_type, filter_list.clone());
                    } else {
                        let mut new_filter_map = std::collections::HashMap::new();
                        new_filter_map.insert(broadcast_type, filter_list.clone());
                        saved_guild.custom_drop_broadcast_filter = Some(new_filter_map);
                    }
                    db.guilds.update_guild(saved_guild).await;
                    None
                } else {
                    Some("Invalid filter.".to_string())
                }
            } else {
                Some("Invalid broadcast type.".to_string())
            };
        }
        Err(_) => {
            return Some(
                "No saved guild was found. Please try adding and removing the bot".to_string(),
            );
        }
    }
}
