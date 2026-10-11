use async_trait::async_trait;
use serenity::all::{CommandDataOption, CreateCommand};
use trackscape_discord_shared::database::BotDb;

#[async_trait]
pub trait _TrackscapeCommand {
    fn register() -> CreateCommand;

    async fn run(
        options: &[CommandDataOption],
        ctx: &serenity::all::Context,
        db: &BotDb,
        guild_id: u64,
    ) -> Option<String>;
}
