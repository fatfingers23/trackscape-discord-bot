mod osrs_broadcast_extractor;
mod ge_api;

use anyhow::anyhow;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_persist::PersistInstance;
use shuttle_secrets::SecretStore;
use tracing::{info};
use crate::ge_api::ge_api::{GeItemMapping, get_item_mapping, get_item_value_by_id};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::{ClanMessage, extract_message};

struct Bot {
    channel_to_check: u64,
    channel_to_send: u64,
    drop_price_threshold: u64,
    persist: PersistInstance,
}


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message)
    {
        //in game chat channel
        if msg.channel_id == self.channel_to_check {
            info!("New message!\n");
            if msg.embeds.iter().count() > 0 {
                let author = msg.embeds[0].author.as_ref().unwrap().name.clone();
                let message = msg.embeds[0].description.as_ref().unwrap().clone();
                let clan_message = ClanMessage {
                    author,
                    message: message.clone(),
                };
                if clan_message.author == "Insomniacs" {
                    let item_mapping_from_state = self.persist
                        .load::<GeItemMapping>("mapping")
                        .map_err(|e| info!("Saving Item Mapping Error: {e}"));
                    let possible_response = extract_message(clan_message, item_mapping_from_state).await;
                    match possible_response {
                        None => {}
                        Some(response) => {
                            //Achievement Channel Id
                            info!("{}\n", message.clone());

                            if response.item_value.is_some() {
                                if response.item_value.unwrap() < self.drop_price_threshold as i64 {
                                    info!("The Item value is less than threshold, not sending message\n");
                                    return;
                                }
                            }

                            let channel = ctx.http.get_channel(self.channel_to_send).await.unwrap();
                            channel.id().send_message(&ctx.http, |m| {
                                m.embed(|e| {
                                    e.title(response.title)
                                        .description(response.message)
                                        .color(0x0000FF);
                                    match response.icon_url {
                                        None => {}
                                        Some(icon_url) => {
                                            e.image(icon_url);
                                        }
                                    }
                                    e
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
    #[shuttle_persist::Persist] persist: PersistInstance
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`as
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let in_game_channel = if let Some(token) = secret_store.get("IN_GAME_CHANNEL") {
        token
    } else {
        return Err(anyhow!("'IN_GAME_CHANNEL' was not found").into());
    };

    let channel_to_send_message_to = if let Some(token) = secret_store.get("CHANNEL_TO_SEND_MESSAGES_TO") {
        token
    } else {
        return Err(anyhow!("'CHANNEL_TO_SEND_MESSAGES_TO' was not found").into());
    };

    let drop_price_threshold = if let Some(token) = secret_store.get("DROP_PRICE_THRESHOLD") {
        token
    } else {
        return Err(anyhow!("'DROP_PRICE_THRESHOLD' was not found").into());
    };

    let ge_mapping_request = get_item_mapping().await;
    match ge_mapping_request {
        Ok(ge_mapping) => {
            let _state = persist
                .save::<GeItemMapping>(
                    "mapping",
                    ge_mapping.clone(),
                )
                .map_err(|e| info!("Saving Item Mapping Error: {e}"));
        }
        Err(error) => {
            info!("Error getting ge mapping: {}", error)
        }
    }



    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot{
            channel_to_check: in_game_channel.parse::<u64>().unwrap(),
            channel_to_send: channel_to_send_message_to.parse::<u64>().unwrap(),
            drop_price_threshold: drop_price_threshold.parse::<u64>().unwrap(),
            persist
        })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
