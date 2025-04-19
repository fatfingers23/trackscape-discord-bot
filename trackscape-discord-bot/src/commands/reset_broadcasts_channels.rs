use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommandOption,
};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

pub fn register() -> CreateCommand {
    CreateCommand::new("reset_broadcasts_channels")
        .description("Resets the broadcast channel for a broadcast type to the default.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "broadcast",
                "Broadcast type to reset the channel back to default.",
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
                BroadcastType::ClueItem.to_string(),
                BroadcastType::ClueItem.to_slug(),
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
                BroadcastType::Pk.to_string(), 
                BroadcastType::Pk.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CofferDonation.to_string(), 
                BroadcastType::CofferDonation.to_slug(),
            )
            .add_string_choice(
                BroadcastType::Invite.to_string(),
                BroadcastType::Invite.to_slug(),
            )
            .add_string_choice(
                BroadcastType::ExpelledFromClan.to_string(),
                 BroadcastType::ExpelledFromClan.to_slug(),
            )
            .add_string_choice(
                BroadcastType::LeftTheClan.to_string(),
                BroadcastType::LeftTheClan.to_slug(),
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
                BroadcastType::CofferDonation.to_string(),
                BroadcastType::CofferDonation.to_slug(),
            )
            .add_string_choice(
                BroadcastType::CofferWithdrawal.to_string(),
                BroadcastType::CofferWithdrawal.to_slug(),
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

            return if let CommandDataOptionValue::String(broadcast_type) =
                broadcast_type.clone().value
            {
                let broadcast_type = BroadcastType::from_string(broadcast_type.replace("_", " "));
                if let Some(ref mut channels) = saved_guild.specific_broadcast_channels {
                    if channels.remove(&broadcast_type).is_some() {
                    } else {
                        return Some("Invalid broadcast type.".to_string());
                    }
                } else {
                    return Some("No specific broadcast channels found.".to_string());
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
