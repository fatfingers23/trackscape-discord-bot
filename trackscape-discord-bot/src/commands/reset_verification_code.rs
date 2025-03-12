use crate::database::BotMongoDb;
use serenity::all::{CommandDataOption, CreateCommand};
use serenity::client::Context;
use serenity::model::prelude::Permissions;

pub fn register() -> CreateCommand {
    CreateCommand::new("reset_verification_code")
        .description("Resets the verification code. MUST UPDATE THE NEW CODE IN THE PLUGIN.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
}

pub async fn run(
    _options: &[CommandDataOption],
    _ctx: &Context,
    db: &BotMongoDb,
    guild_id: u64,
) -> Option<String> {
    let reset_code = db.guilds.reset_verification_code(guild_id).await;
    match reset_code {
        Ok(new_code) => Some(format!("The new verification code is: {}", new_code)),
        Err(e) => {
            eprintln!("Failed to reset verification code: {:?}", e);
            Some("Failed to reset verification code.".to_string())
        }
    }
}
