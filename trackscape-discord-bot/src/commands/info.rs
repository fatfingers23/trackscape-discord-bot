use crate::on_boarding_message::on_boarding_embed;
use crate::{Context, Error};

/// Displays info about the bot
#[poise::command(slash_command)]
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default().embed(on_boarding_embed()))
        .await?;
    Ok(())
}
