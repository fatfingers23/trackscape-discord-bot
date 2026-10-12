use crate::database::BotDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommandOption,
};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

pub fn register() -> CreateCommand {
    CreateCommand::new("wom")
        .description("Connects Trackscape to your Wise Old Man group.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "group_id",
                "Your Wise Old Man group id. Found on the side bar of your group page.",
            )
            .required(true),
        )
}

pub async fn run(
    command: &Vec<CommandDataOption>,
    _ctx: &Context,
    db: &BotDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;
    match saved_guild_query {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let wom_id_command_option = command.get(0).expect("Expected a command option");

            return if let CommandDataOptionValue::Integer(wom_id) = wom_id_command_option.value {
                let wom_client = trackscape_discord_shared::wom::get_wom_client();
                match wom_client.group_client.get_group_details(wom_id).await {
                    Ok(details) => {
                        saved_guild.wom_id = Some(wom_id);
                        db.guilds.update_guild(saved_guild).await;
                        Some(format!(
                            "Connected to the Wise Old Man group \"{}\" ({} members).",
                            details.name, details.member_count
                        ))
                    }
                    Err(_) => Some(format!(
                        "Couldn't find a Wise Old Man group with the id {}. Double-check the group id on your group's page at wiseoldman.net.",
                        wom_id
                    )),
                }
            } else {
                Some("Invalid Wise Old Man Id.".to_string())
            };
        }
        Err(_) => {
            return Some(
                "No saved guild was found. Please try adding and removing the bot".to_string(),
            );
        }
    }
}
