use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption,
};
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use tracing::info;
use trackscape_discord_shared::database::clan_mates::ClanMates;

pub fn register() -> builder::CreateCommand {
    CreateCommand::new("name_change")
        .description("Changes a clanmates name.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "old",
                "The old name of the clanmate. Case sensitive.",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "new",
                "The new name of the clanmate. Case sensitive.",
            )
            .required(true),
        )
}

pub async fn run(
    options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let old_name = options.get(0).expect("Expected string option");
    let new_name = options.get(1).expect("Expected string option");

    if let CommandDataOptionValue::String(rsn) = old_name.clone().value {
        if let CommandDataOptionValue::String(new_rsn) = new_name.clone().value {
            let result = db
                .clan_mates
                .change_name(
                    guild_id,
                    rsn.replace(" ", "\u{a0}"),
                    new_rsn.replace(" ", "\u{a0}"),
                )
                .await;
            return match result {
                Ok(_) => Some(format!("Successfully changed {} to {}.", rsn, new_rsn)),
                Err(_) => Some(format!("Error changing {} to {}.", rsn, new_rsn)),
            };
        }
    }
    info!("Error changing clanmates name.");
    Some("Error expelling the clanmate from TrackScape".to_string())
}
