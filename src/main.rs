mod osrs_broadcast_extractor;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use shuttle_secrets::SecretStore;
use tracing::{error, info};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{BroadcastMessageToDiscord, ClanMessage, extract_message};

struct Bot;


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message)
    {
        //in game chat channel
        if msg.channel_id == 894635400432341043 {
            println!("New message!");
            if msg.embeds.iter().count() > 0 {
                let author = msg.embeds[0].author.as_ref().unwrap().name.clone();
                let message = msg.embeds[0].description.as_ref().unwrap().clone();
                let clan_message = ClanMessage {
                    author,
                    message,
                };
                if clan_message.author == "Insomniacs" {
                    let possible_response = extract_message(clan_message);
                    match possible_response {
                        None => {}
                        Some(response) => {
                            //Achievement Channel Id
                            let channel = ctx.http.get_channel(1147679448993370182).await.unwrap();
                            channel.id().send_message(&ctx.http, |m| {
                                m.embed(|e| {
                                    e.author(|a| a.icon_url(response.icon_url.unwrap()).name("Insomniacs"))
                                        .description(response.message)
                                })
                            }).await.unwrap();

                        }
                    }
                }
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
