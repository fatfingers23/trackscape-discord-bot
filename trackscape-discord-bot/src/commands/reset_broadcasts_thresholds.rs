use crate::commands::choices::ResetBroadcast;
use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

/// Resets a selected broadcast threshold or min broadcast level.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "reset"
)]
pub async fn reset_broadcasts_thresholds(
    ctx: Context<'_>,
    #[description = "Broadcast type to reset notifications back to default."]
    broadcast: ResetBroadcast,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            match broadcast {
                ResetBroadcast::ItemDrop => saved_guild.drop_price_threshold = None,
                ResetBroadcast::Pk => saved_guild.pk_value_threshold = None,
                ResetBroadcast::Quest => saved_guild.min_quest_difficulty = None,
                ResetBroadcast::Diary => saved_guild.min_diary_tier = None,
                ResetBroadcast::CollectionLog => saved_guild.collection_log_max_percentage = None,
            }
            db.guilds.update_guild(saved_guild).await;
            format!(
                "Successfully reset {} broadcast back to default.",
                BroadcastType::from(broadcast).to_string()
            )
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
