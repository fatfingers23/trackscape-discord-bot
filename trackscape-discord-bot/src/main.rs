mod commands;

use anyhow::anyhow;
use mongodb::Database;
use serenity::async_trait;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::Guild;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use std::collections::HashMap;
use tracing::{error, info};
use trackscape_discord_shared::database;
use trackscape_discord_shared::database::BotMongoDb;

struct Bot {
    mongo_db: BotMongoDb,
    trackscape_base_api: String,
}

#[async_trait]
impl EventHandler for Bot {
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if is_new {
            //This fires if it's a new guild it's been added to
            self.mongo_db.save_new_guild(guild.id.0).await;
            if let Some(guild_system_channel) = guild.system_channel_id {
                guild_system_channel
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Welcome to TrackScape!")
                                .description("Thanks for adding TrackScape to your server! For this to work, make sure to install the TrackScape Connector plugin in RuneLite. This is how TrackScape gets the messages to send in discord.")
                                .image("https://cdn.discordapp.com/attachments/961769668866088970/980601140603412510/220406_Trackscape_Logo-13.png")
                                .field("Features", "* Sends in game clan chat to a discord channel of your choice
* Sends embedded broadcasts of your clan's achievements. Including Pet Drops, High Value ", false)
                                .field("Setup", "`/set_broadcast_channel` and `/set_clan_chat_channel` to make sure you have your channels set up to receive messages from the bot! When you set up either a Clan Chat or Broadcast channel a Code will be given. You will enter this in the settings of the RuneLite plugin.", false)
                                .color(0x0000FF);
                            e
                        })
                    })
                    .await
                    .expect("Not able to send welcome message to system channel.");
            }
            info!(
                "Joined a new Discord Server Id: {} and name {}",
                guild.id.0, guild.name
            );
        }
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
                            map.insert("message", msg.content.clone());
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

        let guild_id = GuildId(1148645741653393408);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    commands::set_clan_chat_channel::register(command)
                })
                .create_application_command(|command| {
                    commands::set_broadcast_channel::register(command)
                })
                .create_application_command(|command| {
                    commands::get_verifcation_code::register(command)
                })
        })
        .await;

        match commands {
            Ok(_) => {}
            Err(e) => {
                error!("Error creating guild commands: {}", e)
            }
        }

        //Use this for global commands
        // let guild_command = Command::create_global_application_command(&ctx.http, |command| {
        //     commands::wonderful_command::register(command)
        // })
        //     .await;
        //
        // println!("I created the following global slash command: {:#?}", guild_command);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // println!("Received command interaction: {:#?}", command);

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
                    commands::get_verifcation_code::run(
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

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::MongoDb(local_uri = "{secrets.MONGO_DB_URL}")] db: Database,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`as

    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    let api_base = if let Some(api_base) = secret_store.get("TRACKSCAPE_API_BASE") {
        api_base
    } else {
        return Err(anyhow!("'TRACKSCAPE_API_BASE' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    let client = Client::builder(&token, intents)
        .event_handler(Bot {
            mongo_db: BotMongoDb::new(db),
            trackscape_base_api: api_base,
        })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
