use crate::commands::choices::FilterBroadcast;
use crate::commands::guild_id;
use crate::{Context, Error};
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

/// Sets a list of filter words to use with the selected broadcast type.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn set_custom_drop_broadcast_filter(
    ctx: Context<'_>,
    #[description = "Broadcast type to set a filter list for."] broadcast: FilterBroadcast,
    #[description = "Enter a comma-separated list to filter broadcasts with"] filter: String,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            let broadcast_type = BroadcastType::from(broadcast);

            let filter_list: Vec<String> =
                filter.split(',').map(|s| s.trim().to_string()).collect();
            saved_guild
                .custom_drop_broadcast_filter
                .get_or_insert_with(std::collections::HashMap::new)
                .insert(broadcast_type.clone(), filter_list.clone());
            db.guilds.update_guild(saved_guild).await;
            // The list is not echoed back, a long one would not fit in a Discord message
            format!(
                "Successfully set the {} filter list with {} {}. Use /get_custom_drop_broadcast_filter to see it.",
                broadcast_type.to_string(),
                filter_list.len(),
                if filter_list.len() == 1 {
                    "entry"
                } else {
                    "entries"
                }
            )
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
