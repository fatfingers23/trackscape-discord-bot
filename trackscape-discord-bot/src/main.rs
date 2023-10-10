mod commands;
mod on_boarding_message;

use crate::on_boarding_message::send_on_boarding;
use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::{Guild, UnavailableGuild};
use serenity::model::id::GuildId;
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tracing::{error, info};
use trackscape_discord_shared::api_web_client::ApiWebClient;
use trackscape_discord_shared::database;
use trackscape_discord_shared::database::BotMongoDb;

struct Bot {
    mongo_db: BotMongoDb,
    trackscape_base_api: String,
    trackscape_api_web_client: ApiWebClient,
    dev_guild_id: Option<u64>,
}

struct ServerCount;

impl TypeMapKey for ServerCount {
    type Value = Arc<AtomicUsize>;
}

#[async_trait]
impl EventHandler for Bot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        info!("Guild: {}", guild.name);
        self.mongo_db.create_if_new_guild(guild.id.0).await;
        let server_count = {
            let data_read = ctx.data.read().await;
            data_read
                .get::<ServerCount>()
                .expect("Expected ServerCount in TypeMap.")
                .clone()
        };
        server_count.fetch_add(1, Ordering::SeqCst);
        let new_server_count = server_count.load(Ordering::SeqCst);
        info!("Server Count: {}", new_server_count.clone());
        self.trackscape_api_web_client
            .send_server_count(new_server_count as i64)
            .await;

        if is_new {
            //This fires if it's a new guild it's been added to
            if let Some(guild_system_channel) = guild.system_channel_id {
                send_on_boarding(guild_system_channel, &ctx).await;
            }
            info!(
                "Joined a new Discord Server Id: {} and name {}",
                guild.id.0, guild.name
            );
        }
    }

    async fn guild_delete(&self, _ctx: Context, incomplete: UnavailableGuild, full: Option<Guild>) {
        if full.is_none() {
            info!("Removed from a guild that we don't have access to")
        } else {
            let full = full.unwrap();
            info!("Removed from the guild: {}", full.name)
        }

        if !incomplete.unavailable {
            self.mongo_db.delete_guild(incomplete.id.0).await;
        }

        let server_count = {
            let data_read = _ctx.data.read().await;
            data_read
                .get::<ServerCount>()
                .expect("Expected ServerCount in TypeMap.")
                .clone()
        };
        server_count.fetch_sub(1, Ordering::SeqCst);
        info!("Server Count: {}", server_count.load(Ordering::SeqCst));
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let possible_guild_id = msg.guild_id;
        if msg.author.bot {
            return;
        }
        match possible_guild_id {
            None => {}
            Some(guild_id) => {
                let guild_id = guild_id.0;
                let guild = self.mongo_db.get_guild_by_discord_id(guild_id).await;
                match guild {
                    Ok(guild) => {
                        if guild.is_none() {
                            return;
                        }
                        let unwrapped_guild = guild.unwrap();
                        if unwrapped_guild.clan_chat_channel.is_none() {
                            return;
                        }
                        if unwrapped_guild.clan_chat_channel.unwrap() == msg.channel_id.0 {
                            let mut map = HashMap::new();
                            let mut shortened_message = msg.content.clone();
                            // 78 is the max length of a message in game
                            shortened_message.truncate(78);
                            map.insert("message", shortened_message);
                            let nick_name = msg.author_nick(&ctx.http).await;
                            if let None = nick_name {
                                map.insert("sender", msg.author.name);
                            } else {
                                map.insert("sender", nick_name.unwrap());
                            }

                            let client = reqwest::Client::new();

                            let resp = client
                                .post(
                                    format!(
                                        "{}{}",
                                        self.trackscape_base_api, "/api/chat/new-discord-message"
                                    )
                                    .as_str(),
                                )
                                .header("verification-code", unwrapped_guild.verification_code)
                                .json(&map)
                                .send()
                                .await;
                            if resp.is_err() {
                                error!(
                                    "Error sending message to api: {}",
                                    resp.err().expect("Error getting a error from the error for an api call for new discord chat")
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error getting guild: {}", e)
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        if self.dev_guild_id.is_some() {
            create_commands_for_guild(&GuildId(self.dev_guild_id.unwrap()), ctx.clone()).await;
        } else {
            let global_guild_commands =
                Command::set_global_application_commands(&ctx.http, |commands| {
                    commands
                        .create_application_command(|command| {
                            commands::set_clan_chat_channel::register(command)
                        })
                        .create_application_command(|command| {
                            commands::set_broadcast_channel::register(command)
                        })
                        .create_application_command(|command| {
                            commands::get_verification_code::register(command)
                        })
                        .create_application_command(|command| commands::info::register(command))
                        .create_application_command(|command| {
                            commands::set_threshold_command::register(command)
                        })
                        .create_application_command(|command| {
                            commands::set_quest_min_command::register(command)
                        })
                        .create_application_command(|command| {
                            commands::set_diary_min_command::register(command)
                        })
                        .create_application_command(|command| {
                            commands::reset_broadcasts_thresholds::register(command)
                        })
                        .create_application_command(|command| {
                            commands::toggle_broadcasts::register(command)
                        })
                })
                .await;

            match global_guild_commands {
                Ok(_) => {}
                Err(e) => {
                    error!("Error creating global commands: {}", e)
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            let content = match command.data.name.as_str() {
                "set_clan_chat_channel" => {
                    commands::set_clan_chat_channel::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "set_broadcast_channel" => {
                    commands::set_broadcast_channel::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "get_verification_code" => {
                    commands::get_verification_code::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "info" => {
                    commands::info::run(&command.data.options, &ctx, command.channel_id).await
                }
                "threshold" => {
                    commands::set_threshold_command::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "quests" => {
                    commands::set_quest_min_command::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "diaries" => {
                    commands::set_diary_min_command::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "reset" => {
                    commands::reset_broadcasts_thresholds::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                "toggle" => {
                    commands::toggle_broadcasts::run(
                        &command.data.options,
                        &ctx,
                        &self.mongo_db,
                        command.guild_id.unwrap().0,
                    )
                    .await
                }
                _ => {
                    info!("not implemented :(");
                    None
                }
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| match content {
                    None => response.interaction_response_data(|message| {
                        message.content("Command Completed Successfully.")
                    }),
                    Some(reply) => response.interaction_response_data(|message| {
                        message.content(reply).ephemeral(true)
                    }),
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

pub async fn create_commands_for_guild(guild_id: &GuildId, ctx: Context) {
    let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
        commands
            .create_application_command(|command| {
                commands::set_clan_chat_channel::register(command)
            })
            .create_application_command(|command| {
                commands::set_broadcast_channel::register(command)
            })
            .create_application_command(|command| {
                commands::get_verification_code::register(command)
            })
            .create_application_command(|command| commands::info::register(command))
            .create_application_command(|command| {
                commands::set_threshold_command::register(command)
            })
            .create_application_command(|command| {
                commands::set_quest_min_command::register(command)
            })
            .create_application_command(|command| {
                commands::set_diary_min_command::register(command)
            })
            .create_application_command(|command| {
                commands::reset_broadcasts_thresholds::register(command)
            })
            .create_application_command(|command| commands::toggle_broadcasts::register(command))
    })
    .await;
    match commands {
        Ok(_) => {}
        Err(e) => {
            error!("Error creating guild commands: {}, for: {}", e, guild_id)
        }
    }
}

#[shuttle_runtime::main]
async fn serenity() -> shuttle_serenity::ShuttleSerenity {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    let api_base = env::var("TRACKSCAPE_API_BASE").expect("TRACKSCAPE_API_BASE not set!");
    let mongodb_url = env::var("MONGO_DB_URL").expect("MONGO_DB_URL not set!");
    let trackscape_api_token = env::var("MANAGEMENT_API_KEY").expect("MANAGEMENT_API_KEY not set!");
    let dev_guild_id = match env::var("DEV_GUILD_ID") {
        Ok(id) => Some(id.parse::<u64>().expect("DEV_GUILD_ID is not a number")),
        Err(_) => None,
    };

    let db = BotMongoDb::new_db_instance(mongodb_url).await;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;
    let api_client = ApiWebClient::new(api_base.clone(), trackscape_api_token);
    let client = serenity::Client::builder(&token, intents)
        .event_handler(Bot {
            mongo_db: db,
            trackscape_base_api: api_base,
            trackscape_api_web_client: api_client,
            dev_guild_id,
        })
        .await
        .expect("Err creating client");
    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<ServerCount>(Arc::new(AtomicUsize::new(0)));
    }
    Ok(client.into())
}
