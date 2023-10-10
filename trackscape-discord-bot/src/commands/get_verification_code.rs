use crate::database::BotMongoDb;

use serenity::builder;
use serenity::client::Context;

use serenity::model::prelude::application_command::CommandDataOption;

use serenity::model::prelude::Permissions;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("get_verification_code")
        .description("Gets the verification code for the server to be used in the RuneLite plugin.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
}

pub async fn run(
    _options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let saved_guild_query = db.guilds.get_by_guild_id(guild_id).await;

    return match saved_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(saved_guild) => {
                let saved_guild_code = saved_guild.verification_code.clone();
                Some(format!("The clan's verification code is: `{}`", saved_guild_code))
            }
            None => {
                Some("Error finding your server as registered. Try kicking and re adding the bot please.".to_string())
            }
        },
        Err(_) => {
            Some("There was a technical error. Please try again later.".to_string())
        }
    };
}
