use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType::ItemDrop;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("reset")
        .description("Resets a selected broadcast threshold or min broadcast level.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("broadcast")
                .description("Broadcast type to reset notifications back to default.")
                .kind(CommandOptionType::String)
                .add_string_choice(BroadcastType::ItemDrop.to_string(), ItemDrop.to_slug())
                .add_string_choice(BroadcastType::Pk.to_string(), BroadcastType::Pk.to_slug())
                .add_string_choice(
                    BroadcastType::Quest.to_string(),
                    BroadcastType::Quest.to_slug(),
                )
                .add_string_choice(
                    BroadcastType::Diary.to_string(),
                    BroadcastType::Diary.to_slug(),
                )
                .required(true)
        })
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
            let broadcast_type = command
                .get(0)
                .expect("Expected broadcast type option")
                .resolved
                .as_ref()
                .expect("Expected broadcast type object");

            return if let CommandDataOptionValue::String(broadcast_type) = broadcast_type {
                let broadcast_type = BroadcastType::from_string(broadcast_type.replace("_", " "));
                match broadcast_type {
                    BroadcastType::ItemDrop => {
                        saved_guild.drop_price_threshold = None;
                    }
                    BroadcastType::Pk => {
                        saved_guild.pk_value_threshold = None;
                    }
                    BroadcastType::Quest => {
                        saved_guild.min_quest_difficulty = None;
                    }
                    BroadcastType::Diary => {
                        saved_guild.min_diary_tier = None;
                    }
                    _ => {
                        return Some("Invalid broadcast type.".to_string());
                    }
                }
                db.guilds.update_guild(saved_guild).await;
                Some(format!(
                    "Successfully reset {} broadcast back to default.",
                    broadcast_type.to_string()
                ))
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
