use crate::database::BotMongoDb;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateCommand,
    CreateCommandOption,
};
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

pub fn register() -> CreateCommand {
    CreateCommand::new("threshold")
        .description("Sets min amount to trigger a broadcast with GP amounts.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "broadcast",
                "Broadcast type to configure threshold for.",
            )
            .add_string_choice("Item Drops", "item_drop")
            .add_string_choice("PK Loot", "pk_loot")
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "threshold",
                "The minimal drop gp value to broadcast.",
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

            let threshold = command.get(1).expect("Expected threshold option");

            return if let CommandDataOptionValue::String(broadcast_type) =
                broadcast_type.clone().value
            {
                if let CommandDataOptionValue::Integer(threshold) = threshold.value {
                    match broadcast_type.as_str() {
                        "item_drop" => {
                            saved_guild.drop_price_threshold = Some(threshold);
                            db.guilds.update_guild(saved_guild).await;
                            None
                        }
                        "pk_loot" => {
                            //TODO: Implement
                            saved_guild.pk_value_threshold = Some(threshold);
                            db.guilds.update_guild(saved_guild).await;
                            None
                        }
                        _ => Some("Invalid broadcast type.".to_string()),
                    }
                } else {
                    Some("Invalid threshold.".to_string())
                }
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
