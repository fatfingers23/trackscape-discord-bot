use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::DiaryTier;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("diaries")
        .description("Sets min diary tier to trigger a broadcast. Anything below will not send")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("tier")
                .description("Min diary tier difficulty to send a broadcast.")
                .kind(CommandOptionType::String)
                .add_string_choice(DiaryTier::Easy.to_string(), DiaryTier::Easy.to_string())
                .add_string_choice(DiaryTier::Medium.to_string(), DiaryTier::Medium.to_string())
                .add_string_choice(DiaryTier::Hard.to_string(), DiaryTier::Hard.to_string())
                .add_string_choice(DiaryTier::Elite.to_string(), DiaryTier::Elite.to_string())
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
            let possible_diary_tier = command
                .get(0)
                .expect("Expected diary tier option")
                .resolved
                .as_ref()
                .expect("Expected diary tier object");

            return if let CommandDataOptionValue::String(diary_tier) = possible_diary_tier {
                saved_guild.min_diary_tier =
                    Some(DiaryTier::from_string(diary_tier.clone().to_string()));
                db.guilds.update_guild(saved_guild).await;
                Some("Successfully updated min diary to broadcast.".to_string())
            } else {
                Some("Invalid option.".to_string())
            };
        }
        Err(_) => {
            return Some(
                "No saved guild was found. Please try adding and removing the bot".to_string(),
            );
        }
    }
}
