use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::RegisteredGuild;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType::ItemDrop;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("toggle")
        .description("Turns on or off a broadcast type to be sent.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("broadcast")
                .description("Broadcast type to toggle on or off.")
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
                    BroadcastType::LevelMilestone.to_string(),
                    BroadcastType::LevelMilestone.to_slug(),
                )
                .add_string_choice(
                    BroadcastType::XPMilestone.to_string(),
                    BroadcastType::XPMilestone.to_slug(),
                )
                .required(true)
        })
        .create_option(|option| {
            option
                .name("toggle")
                .description("Turns on(true) or off(false) the broadcast type.")
                .kind(CommandOptionType::Boolean)
                .required(true)
        })
}

pub async fn run(
    command: &Vec<CommandDataOption>,
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.get_by_guild_id(guild_id).await;
    match saved_guild_query {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuild::new(guild_id));
            let broadcast_type = command
                .get(0)
                .expect("Expected broadcast type option")
                .resolved
                .as_ref()
                .expect("Expected broadcast type object");

            let toggle = command
                .get(1)
                .expect("Expected toggle option")
                .resolved
                .as_ref()
                .expect("Expected toggle object");

            return if let CommandDataOptionValue::String(broadcast_type) = broadcast_type {
                return if let CommandDataOptionValue::Boolean(toggle) = toggle {
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

                    db.update_guild(saved_guild).await;
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
