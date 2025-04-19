use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommandOption,
};
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

pub fn register() -> CreateCommand {
    CreateCommand::new("set_clog_max_percentage")
        .description(
            "Set the max completion percentage for allowed broadcasts (see wiki collection log table)",
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "percentage",
                "Collection log max completion %, max 100",
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
            let possible_percentage = command.get(0).expect("Expected percentage option");

            return if let CommandDataOptionValue::String(percentage) =
                possible_percentage.clone().value
            {
                saved_guild.collection_log_max_percentage = match percentage.parse::<f64>() {
                    Ok(value) if value <= 100.0 && value >= 0.0 => Some(value),
                    Ok(_) => return Some("Percentage must be between 0 and 100.".to_string()),
                    Err(_) => return Some("Invalid percentage value. Please provide a valid number.".to_string()),
                };
                db.guilds.update_guild(saved_guild).await;
                Some("Successfully updated the Collection Log max percentage.".to_string())
            } else {
                Some("Invalid threshold.".to_string())
            };
        }
        Err(_) => {
            return Some(
                "No saved guild was found. Please try adding and removing the bot".to_string(),
            );
        }
    }
}
