use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("wom")
        .description("Connects Trackscape to your Wise Old Man group.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("group_id")
                .description(
                    "Your Wise Old Man group id. Found on the side bar of your group page.",
                )
                .kind(CommandOptionType::Integer)
                .required(true)
        })
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
            let wom_id_command_option = command
                .get(0)
                .expect("Expected a command option")
                .resolved
                .as_ref()
                .expect("Expected a command option");

            return if let CommandDataOptionValue::Integer(wom_id) = wom_id_command_option {
                //TODO when we have a wom client check if its a valid id
                saved_guild.wom_id = Some(*wom_id);
                let _ = db.guilds.update_guild(saved_guild).await;
                None
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
