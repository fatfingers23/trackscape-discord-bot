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
    CreateCommand::new("expel")
        .description("Removes a clanmate from the clan in TrackScape.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "rsn",
                "Player to remove from the clan. Case sensitive.",
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
    let option = options.get(0).expect("Expected string option");

    if let CommandDataOptionValue::String(rsn) = option.clone().value {
        let result = db
            .clan_mates
            .remove_clan_mate(guild_id, rsn.replace(" ", "\u{a0}"))
            .await;
        return match result {
            Ok(_) => Some(format!("Successfully removed {} from the clan.", rsn)),
            Err(_) => Some(format!("Error removing {} from the clan.", rsn)),
        };
    }
    info!("Error removing the clanmate.");
    Some("Error expelling the clanmate from TrackScape".to_string())
}
