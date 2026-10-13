use crate::database::BotDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption,
};
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use tracing::{error, info};
use trackscape_discord_shared::database::clan_mates::ClanMates;

pub fn register() -> builder::CreateCommand {
    CreateCommand::new("expel")
        .description("Removes a clanmate from the clan in TrackScape.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "rsn",
                "Player to mark as having left the clan.",
            )
            .required(true),
        )
}

pub async fn run(
    options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotDb,
    guild_id: u64,
) -> Option<String> {
    let option = options.get(0).expect("Expected string option");

    if let CommandDataOptionValue::String(rsn) = option.clone().value {
        // No Redis here: the chat job's cached copy may delay a chat-driven restore by up to an hour.
        let result = db
            .clan_mates
            .soft_remove_clan_mate(guild_id, rsn.clone())
            .await;
        return match result {
            Ok(_) => Some(format!("Marked {} as having left the clan.", rsn)),
            Err(err) => {
                error!("Error marking {} as having left the clan: {:?}", rsn, err);
                if err.to_string().contains("Failed to find clan mate") {
                    Some(format!(
                        "Could not find {} in this clan's TrackScape members.",
                        rsn
                    ))
                } else {
                    Some(format!("Error marking {} as having left the clan.", rsn))
                }
            }
        };
    }
    info!("Error removing the clanmate.");
    Some("Error expelling the clanmate from TrackScape".to_string())
}
