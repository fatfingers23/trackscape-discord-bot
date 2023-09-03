mod osrs_broadcast_extractor;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::ClanMessage;

struct Bot;


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message)
    {
        if msg.channel_id == 894635400432341043 {
            println!("New message!");
            msg.embeds.iter().for_each(|embed| {
                let author = embed.author.as_ref().unwrap().name.clone();
                let message = embed.description.as_ref().unwrap().clone();
                let clan_message = ClanMessage {
                    author,
                    message,
                };
                println!("Author: {} Message: {}", clan_message.author, clan_message.message);

            });

        }
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`as
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
