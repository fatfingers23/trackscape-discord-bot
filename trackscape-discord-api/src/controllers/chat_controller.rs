use crate::cache::Cache;
use crate::websocket_server::DiscordToClanChatMessage;
use crate::{handler, ChatServerHandle};
use actix_web::{error, post, web, Error, HttpRequest, HttpResponse, Scope};
use log::info;
use serenity::builder::CreateMessage;
use serenity::http::Http;
use serenity::json;
use serenity::json::Value;
use shuttle_persist::PersistInstance;
use tokio::task::spawn_local;
use trackscape_discord_shared::database::BotMongoDb;
use trackscape_discord_shared::ge_api::ge_api::GeItemMapping;
use trackscape_discord_shared::helpers::hash_string;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    get_wiki_clan_rank_image_url, ClanMessage,
};

use trackscape_discord_shared::osrs_broadcast_handler::OSRSBroadcastHandler;
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
        //HACK temp code till plugin is updated
        let message_clone_to_check_join_leave = chat.message.clone();
        if message_clone_to_check_join_leave.contains("has joined.")
            || message_clone_to_check_join_leave.contains("has left.")
        {
            continue;
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

        if let None = registered_guild.clan_name {
            registered_guild.clan_name = Some(chat.clan_name.clone());
            mongodb.update_guild(registered_guild.clone()).await
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

        if let Some(broadcast_channel_id) = registered_guild.broadcast_channel {
            let item_mapping_from_state = persist
                .load::<GeItemMapping>("mapping")
                .map_err(|e| info!("Saving Item Mapping Error: {e}"));
            let quests_from_state = persist
                .load::<Vec<WikiQuest>>("quests")
                .map_err(|e| info!("Saving Quests Error: {e}"));
            let handler = OSRSBroadcastHandler::new(
                chat.clone(),
                item_mapping_from_state,
                quests_from_state,
                registered_guild.clone(),
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

                    let map = json::hashmap_to_json_map(create_message.0);
                    discord_http_client
                        .send_message(broadcast_channel_id, &Value::from(map))
                        .await
                        .unwrap();
                }
            };
        }
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
