use crate::database::BotMongoDb;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand, CreateCommandOption};
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

pub fn register() -> CreateCommand {
    CreateCommand::new("get_custom_drop_broadcast_filter")
        .description("Gets the filter list being used to filter broadcasts for the selected broadcast type.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "broadcast",
                "Broadcast type to return the filter list for.",
            )
            .add_string_choice(
                BroadcastType::ItemDrop.to_string(), 
                BroadcastType::ItemDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::PetDrop.to_string(),
                BroadcastType::PetDrop.to_slug(),
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
            .required(true),
        )
}

pub async fn run(
    _options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    match saved_guild_query {
        Ok(saved_guild) => {
            let saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let broadcast_type = _options.get(0).expect("Expected a broadcast type option");
            return if let CommandDataOptionValue::String(broadcast_type) = broadcast_type.clone().value{
                let broadcast_type = 
                BroadcastType::from_string(broadcast_type.replace("_", " "));
            
                if let Some(ref filter_map) = saved_guild.custom_drop_broadcast_filter {
                    if let Some(filter_list) = filter_map.get(&broadcast_type) {
                        Some(format!(
                            "The current {} filter list is: `{}`",
                            broadcast_type.to_string(),
                            filter_list.join(", ")
                        ))
                    } else {
                        Some(format!(
                            "No filter list found for {}. Please set a new filter list.",
                            broadcast_type.to_string()
                        ))
                    }
                } else {
                    Some("Error finding filter lists, try again later.".to_string())
                }
            } else {
                Some("Invalid Broadcast Type".to_string())
            };
        }
        Err(_) => {
            return Some("There was a technical error. Please try again later.".to_string(),
            );
        }
            
    }
}
