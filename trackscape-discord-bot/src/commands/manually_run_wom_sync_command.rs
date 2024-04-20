use crate::database::BotMongoDb;
use serenity::builder::CreateCommand;
use serenity::client::Context;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::jobs::wom_guild_sync_logic::sync_wom_by_guild;

pub fn register() -> CreateCommand {
    CreateCommand::new("sync")
        .description("If WOM is connected it syncs between Trackscape and WOM.")
        .default_member_permissions(Permissions::MANAGE_GUILD)
}

pub async fn run(_ctx: &Context, db: &BotMongoDb, guild_id: u64) -> Option<String> {
    match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            match saved_guild.wom_id {
                Some(_) => {
                    sync_wom_by_guild(&saved_guild, db).await;
                    None
                }
                None => Some("Please set your WOM group's id first with /wom ".to_string()),
            }
        }
        Err(_) => {
            Some("No saved guild was found. Please try adding and removing the bot".to_string())
        }
    }
}
