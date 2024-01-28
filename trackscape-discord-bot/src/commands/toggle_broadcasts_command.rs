use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommandOption,
};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType::ItemDrop;

pub fn register() -> CreateCommand {
    CreateCommand::new("toggle")
        .description("Turns on or off a broadcast type to be sent.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "broadcast",
                "Broadcast type to toggle on or off.",
            )
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
            .add_string_choice(
                BroadcastType::RaidDrop.to_string(),
                BroadcastType::RaidDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::PetDrop.to_string(),
                BroadcastType::PetDrop.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Invite.to_string(),
                BroadcastType::Invite.to_slug(),
            )
            .add_string_choice(
                BroadcastType::LeftTheClan.to_string(),
                BroadcastType::LeftTheClan.to_slug(),
            )
            .add_string_choice(
                BroadcastType::ExpelledFromClan.to_string(),
                BroadcastType::ExpelledFromClan.to_slug(),
            )
            .add_string_choice(
                BroadcastType::LevelMilestone.to_string(),
                BroadcastType::LevelMilestone.to_slug(),
            )
            .add_string_choice(
                BroadcastType::XPMilestone.to_string(),
                BroadcastType::XPMilestone.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CollectionLog.to_string(),
                BroadcastType::CollectionLog.to_slug(),
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Boolean,
                "toggle",
                "Turns on(true) or off(false) the broadcast type.",
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
            let broadcast_type = command.get(0).expect("Expected broadcast type option");

            let toggle = command.get(1).expect("Expected toggle option");

            return if let CommandDataOptionValue::String(broadcast_type) =
                broadcast_type.clone().value
            {
                return if let CommandDataOptionValue::Boolean(toggle) = toggle.clone().value {
                    let broadcast_type =
                        BroadcastType::from_string(broadcast_type.replace("_", " "));
                    if toggle.clone() {
                        saved_guild.disallowed_broadcast_types = saved_guild
                            .disallowed_broadcast_types
                            .iter()
                            .filter(|&b| b != &broadcast_type)
                            .cloned()
                            .collect();
                    } else {
                        saved_guild
                            .disallowed_broadcast_types
                            .push(broadcast_type.clone());
                    }

                    db.guilds.update_guild(saved_guild).await;
                    if toggle.clone() {
                        return Some(format!(
                            "Successfully set to send {} broadcasts.",
                            broadcast_type.to_string()
                        ));
                    } else {
                        return Some(format!(
                            "Successfully set to NOT send {} broadcasts.",
                            broadcast_type.to_string()
                        ));
                    }
                } else {
                    Some("Invalid toggle type.".to_string())
                };
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
