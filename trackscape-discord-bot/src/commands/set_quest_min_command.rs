use crate::database::BotMongoDb;
use serenity::builder;
use serenity::client::Context;
use serenity::model::prelude::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::Permissions;
use trackscape_discord_shared::database::RegisteredGuild;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::QuestDifficulty;

pub fn register(
    command: &mut builder::CreateApplicationCommand,
) -> &mut builder::CreateApplicationCommand {
    command
        .name("quests")
        .description(
            "Sets min quest difficulty to trigger a broadcast. Anything below will not send",
        )
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("difficulty")
                .description("Min quest difficulty to send a broadcast.")
                .kind(CommandOptionType::String)
                .add_string_choice(
                    QuestDifficulty::Novice.to_string(),
                    QuestDifficulty::Novice.to_string(),
                )
                .add_string_choice(
                    QuestDifficulty::Intermediate.to_string(),
                    QuestDifficulty::Intermediate.to_string(),
                )
                .add_string_choice(
                    QuestDifficulty::Experienced.to_string(),
                    QuestDifficulty::Experienced.to_string(),
                )
                .add_string_choice(
                    QuestDifficulty::Master.to_string(),
                    QuestDifficulty::Master.to_string(),
                )
                .add_string_choice(
                    QuestDifficulty::Grandmaster.to_string(),
                    QuestDifficulty::Grandmaster.to_string(),
                )
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
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuild::new(guild_id));
            let possible_broadcast_type = command
                .get(0)
                .expect("Expected broadcast type option")
                .resolved
                .as_ref()
                .expect("Expected broadcast type object");

            return if let CommandDataOptionValue::String(quest_difficulty) = possible_broadcast_type
            {
                saved_guild.min_quest_difficulty = Some(QuestDifficulty::from_string(
                    quest_difficulty.clone().to_string(),
                ));
                db.guilds.update_guild(saved_guild).await;
                Some("Successfully updated min quest difficulty to broadcast.".to_string())
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
