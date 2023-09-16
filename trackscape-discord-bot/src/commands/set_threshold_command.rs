use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::RegisteredGuild;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("threshold")
        .description("Sets min amount to trigger a broadcast with GP amounts.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("broadcast")
                .description("Broadcast type to configure threshold for.")
                .kind(CommandOptionType::String)
                .add_string_choice("Item Drops", "item_drop")
                .add_string_choice("PK Loot", "pk_loot")
                .required(true)
        })
        .create_option(|option| {
            option
                .name("threshold")
                .description("The minimal drop gp value to broadcast.")
                .kind(CommandOptionType::Integer)
                .required(true)
        })
}

pub async fn run(
    command: &Vec<CommandDataOption>,
    ctx: &Context,
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

            let threshold = command
                .get(1)
                .expect("Expected threshold option")
                .resolved
                .as_ref()
                .expect("Expected threshold object");

            return if let CommandDataOptionValue::String(broadcast_type) = broadcast_type {
                if let CommandDataOptionValue::Integer(threshold) = threshold {
                    match broadcast_type.as_str() {
                        "item_drop" => {
                            saved_guild.drop_price_threshold = Some(*threshold);
                            db.update_guild(saved_guild).await;
                            None
                        }
                        "pk_loot" => {
                            //TODO: Implement
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
