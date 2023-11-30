use crate::cache::Cache;
use crate::services::osrs_broadcast_handler::OSRSBroadcastHandler;
use crate::websocket_server::DiscordToClanChatMessage;
use crate::{handler, ChatServerHandle};
use actix_web::web::Data;
use actix_web::{error, post, web, Error, HttpRequest, HttpResponse, Scope};
use celery::error::CeleryError;
use celery::task::AsyncResult;
use celery::Celery;
use log::{error, info};
use serenity::builder::CreateMessage;
use serenity::http::Http;
use serenity::json;
use serenity::json::Value;
use shuttle_persist::PersistInstance;
use std::sync::Arc;
use tokio::task::spawn_local;
use trackscape_discord_shared::database::BotMongoDb;
use trackscape_discord_shared::ge_api::ge_api::GeItemMapping;
use trackscape_discord_shared::helpers::hash_string;
use trackscape_discord_shared::jobs::CeleryJobQueue;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    get_wiki_clan_rank_image_url, ClanMessage,
};
use trackscape_discord_shared::wiki_api::wiki_api::WikiQuest;

#[derive(Debug)]
struct MyError {
    message: &'static str,
}

#[post("/new-discord-message")]
async fn new_discord_message(
    req: HttpRequest,
    chat_server: web::Data<ChatServerHandle>,
    new_chat: web::Json<DiscordToClanChatMessage>,
    mongodb: web::Data<BotMongoDb>,
) -> actix_web::Result<String> {
    let possible_verification_code = req.headers().get("verification-code");

    if let None = possible_verification_code {
        let result = Err(MyError {
            message: "No verification code was set",
        });
        return result.map_err(|err| error::ErrorBadRequest(err.message));
    }

    let verification_code = possible_verification_code.unwrap().to_str().unwrap();

    let registered_guild_query = mongodb
        .guilds
        .get_guild_by_code(verification_code.to_string())
        .await;

    let registered_guild_successful_query = if let Ok(registered_guild) = registered_guild_query {
        registered_guild
    } else {
        registered_guild_query.map_err(|err| error::ErrorInternalServerError(err))?
    };

    if let None = registered_guild_successful_query {
        let result = Err(MyError {
            message: "The verification code was not found",
        });
        return result.map_err(|err| error::ErrorBadRequest(err.message));
    }

    let sanitized_message = ammonia::clean(new_chat.message.as_str());
    let sanitized_sender = ammonia::clean(new_chat.sender.as_str());
    chat_server
        .send_discord_message_to_clan_chat(
            sanitized_sender,
            sanitized_message,
            verification_code.to_string(),
        )
        .await;
    Ok("".to_string())
}

#[post("/new-clan-chat")]
async fn new_clan_chats(
    req: HttpRequest,
    discord_http_client: web::Data<Http>,
    cache: web::Data<Cache>,
    new_chat: web::Json<Vec<ClanMessage>>,
    mongodb: web::Data<BotMongoDb>,
    persist: web::Data<PersistInstance>,
    celery: Data<Arc<Celery>>,
) -> actix_web::Result<String> {
    let possible_verification_code = req.headers().get("verification-code");
    if let None = possible_verification_code {
        let result = Err(MyError {
            message: "No verification code was set",
        });
        return result.map_err(|err| error::ErrorBadRequest(err.message));
    }

    let verification_code = possible_verification_code.unwrap().to_str().unwrap();
    //checks to make sure the registered guild exists for the RuneScape clan
    let registered_guild_query = mongodb
        .guilds
        .get_guild_by_code(verification_code.to_string())
        .await;

    let registered_guild_successful_query = if let Ok(registered_guild) = registered_guild_query {
        registered_guild
    } else {
        registered_guild_query.map_err(|err| error::ErrorInternalServerError(err))?
    };

    let mut registered_guild = if let Some(registered_guild) = registered_guild_successful_query {
        registered_guild
    } else {
        let result = Err(MyError {
            message: "The verification code was not found",
        });
        return result.map_err(|err| error::ErrorBadRequest(err.message));
    };

    for chat in new_chat.clone() {
        if chat.sender.clone() == "" && chat.clan_name.clone() == "" {
            continue;
        }

        if chat.is_league_world.is_some() {
            if chat.is_league_world.unwrap() {
                info!("Broadcast from League World")
            }
        }
        //Checks to make sure the message has not already been process since multiple people could be submitting them
        let message_content_hash = hash_string(chat.message.clone());
        match cache.get_value(message_content_hash.clone()).await {
            Some(_) => continue,
            None => {
                cache
                    .set_value(message_content_hash.clone(), "true".to_string())
                    .await;
            }
        }

        //Sets the clan name to the guild. Bit of a hack but best way to get clan name
        if let None = registered_guild.clan_name {
            registered_guild.clan_name = Some(chat.clan_name.clone());
            mongodb.guilds.update_guild(registered_guild.clone()).await
        }

        if registered_guild.clan_name.clone().unwrap() != chat.clan_name {
            continue;
        }

        let right_now = serenity::model::timestamp::Timestamp::now();

        match registered_guild.clan_chat_channel {
            Some(channel_id) => {
                let mut clan_chat_to_discord = CreateMessage::default();
                let author_image = match chat.clan_name.clone() == chat.sender.clone() {
                    true => {
                        "https://oldschool.runescape.wiki/images/Your_Clan_icon.png".to_string()
                    }
                    false => get_wiki_clan_rank_image_url(chat.rank.clone()),
                };

                clan_chat_to_discord.embed(|e| {
                    e.title("")
                        .author(|a| a.name(chat.sender.clone()).icon_url(author_image))
                        .description(chat.message.clone())
                        .color(0x0000FF)
                        .timestamp(right_now)
                });
                let map = json::hashmap_to_json_map(clan_chat_to_discord.0);
                discord_http_client
                    .send_message(channel_id, &Value::from(map))
                    .await
                    .unwrap();
            }
            _ => {}
        }
        //Checks to see if it is a clan broadcast. Clan name and sender are the same if so
        if chat.sender != chat.clan_name {
            //Starts a job to either add the clan mate if not added to guild, or check for rank change
            let _ = celery
                .send_task(
                    trackscape_discord_shared::jobs::update_create_clanmate_job::update_create_clanmate::new(
                        chat.sender.clone(),
                        chat.rank.clone(),
                        registered_guild.guild_id,
                    ),
                )
                .await;
        }

        //TODO may remove this since the handler does some loging for the website now
        if registered_guild.broadcast_channel.is_none()
            && registered_guild.clan_chat_channel.is_none()
        {
            //If there is not any broadcast_channels set just continue
            continue;
        }

        let league_world = chat.is_league_world.unwrap_or(false);

        let item_mapping_from_state = persist
            .load::<GeItemMapping>("mapping")
            .map_err(|e| info!("Saving Item Mapping Error: {e}"));
        let quests_from_state = persist
            .load::<Vec<WikiQuest>>("quests")
            .map_err(|e| info!("Saving Quests Error: {e}"));

        let cloned_celery = Arc::clone(&**celery);
        let celery_job_queue = Arc::new(CeleryJobQueue {
            celery: cloned_celery,
        });

        let handler = OSRSBroadcastHandler::new(
            chat.clone(),
            item_mapping_from_state,
            quests_from_state,
            registered_guild.clone(),
            league_world,
            mongodb.drop_logs.clone(),
            mongodb.clan_mate_collection_log_totals.clone(),
            mongodb.clan_mates.clone(),
            celery_job_queue,
        );
        let possible_broadcast = handler.extract_message().await;

        match possible_broadcast {
            None => {}
            Some(broadcast) => {
                info!("Broadcast: {:?}", broadcast);
                info!("{}\n", chat.message.clone());

                let mut create_message = CreateMessage::default();
                create_message.embed(|e| {
                    e.title(broadcast.title)
                        .description(broadcast.message)
                        .color(0x0000FF)
                        .timestamp(right_now);
                    match broadcast.icon_url {
                        None => {}
                        Some(icon_url) => {
                            e.image(icon_url);
                        }
                    }
                    e
                });

                let possible_channel_to_send_broadcast = match league_world {
                    true => registered_guild.leagues_broadcast_channel,
                    false => registered_guild.broadcast_channel,
                };
                if let Some(channel_to_send_broadcast) = possible_channel_to_send_broadcast {
                    let map = json::hashmap_to_json_map(create_message.0);
                    discord_http_client
                        .send_message(channel_to_send_broadcast, &Value::from(map))
                        .await
                        .unwrap();
                }
            }
        };
    }

    return Ok("Message processed".to_string());
}

/// Handshake and start WebSocket handler with heartbeats.
async fn chat_ws(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<ChatServerHandle>,
    mongodb: web::Data<BotMongoDb>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream).expect("Failed to start WS");
    let possible_verification_code = req.headers().get("verification-code");

    if let None = possible_verification_code {
        let result = MyError {
            message: "No verification code was set",
        };
        return Err(error::ErrorBadRequest(result.message));
    }
    let verification_code = possible_verification_code.unwrap().to_str().unwrap();

    let registered_guild_query = mongodb
        .guilds
        .get_guild_by_code(verification_code.to_string())
        .await;

    let registered_guild_successful_query = if let Ok(registered_guild) = registered_guild_query {
        registered_guild
    } else {
        registered_guild_query.map_err(|err| error::ErrorInternalServerError(err))?
    };

    if let None = registered_guild_successful_query {
        let result = Err(MyError {
            message: "The verification code was not found",
        });
        return result.map_err(|err| error::ErrorBadRequest(err.message));
    }

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(handler::chat_ws(
        (**chat_server).clone(),
        session,
        msg_stream,
        verification_code.to_string(),
    ));

    Ok(res)
}

pub fn chat_controller() -> Scope {
    web::scope("/chat")
        .service(new_clan_chats)
        .service(new_discord_message)
        .service(web::resource("/ws").route(web::get().to(chat_ws)))
}

#[cfg(test)]
mod tests {

    #[test]
    fn sanitize_img() {
        let message = "<img=1> Hello world!";
        let result = ammonia::clean(message);
        assert_eq!(result, " Hello world!");
    }

    #[test]
    fn sanitize_col() {
        let message = "<img=1><col=7320d8>test</col>";
        let result = ammonia::clean(message);
        assert_eq!(result, "test");
    }
}
