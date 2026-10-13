use crate::commands::choices::ThresholdBroadcast;
use crate::commands::guild_id;
use crate::{Context, Error};
use poise::ChoiceParameter;
use trackscape_discord_shared::database::guilds_db::RegisteredGuildModel;

/// Sets min amount to trigger a broadcast with GP amounts.
#[poise::command(
    slash_command,
    guild_only,
    ephemeral,
    default_member_permissions = "MANAGE_GUILD",
    rename = "threshold"
)]
pub async fn set_threshold_command(
    ctx: Context<'_>,
    #[description = "Broadcast type to configure threshold for."] broadcast: ThresholdBroadcast,
    #[description = "The minimal drop gp value to broadcast."] threshold: i64,
) -> Result<(), Error> {
    let guild_id = guild_id(ctx)?;
    let db = &ctx.data().db;

    let reply = match db.guilds.get_by_guild_id(guild_id).await {
        Ok(saved_guild) => {
            let mut saved_guild = saved_guild.unwrap_or(RegisteredGuildModel::new(guild_id));
            match broadcast {
                ThresholdBroadcast::ItemDrop => saved_guild.drop_price_threshold = Some(threshold),
                //TODO: Implement
                ThresholdBroadcast::Pk => saved_guild.pk_value_threshold = Some(threshold),
            }
            db.guilds.update_guild(saved_guild).await;
            format!(
                "Successfully set the {} broadcast threshold to {} gp.",
                broadcast.name(),
                threshold
            )
        }
        Err(_) => "No saved guild was found. Please try adding and removing the bot".to_string(),
    };
    ctx.say(reply).await?;
    Ok(())
}
