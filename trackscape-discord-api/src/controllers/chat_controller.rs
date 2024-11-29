use crate::websocket_server::DiscordToClanChatMessage;
use crate::{handler, ChatServerHandle};
use actix_web::web::Data;
use actix_web::{error, post, web, Error, HttpRequest, HttpResponse, Scope};
use celery::Celery;
use serenity::all::{ChannelId, CreateEmbed, CreateEmbedAuthor};
use serenity::builder::CreateMessage;
use serenity::http::Http;
use std::sync::Arc;
use tokio::task::spawn_local;
use trackscape_discord_shared::database::BotMongoDb;
use trackscape_discord_shared::ge_api::ge_api::get_item_mapping;
use trackscape_discord_shared::helpers::hash_string;
use trackscape_discord_shared::jobs::CeleryJobQueue;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::{
    get_wiki_clan_rank_image_url, ClanMessage,
};
use trackscape_discord_shared::osrs_broadcast_handler::OSRSBroadcastHandler;
use trackscape_discord_shared::redis_helpers::{fetch_redis, write_to_cache_with_seconds};
use trackscape_discord_shared::wiki_api::wiki_api::get_quests_and_difficulties;
use web::Json;

#[derive(Debug)]
struct MyError {
    message: &'static str,
}

const LEAGUES_ICON_TAG: &str = "<img=22> ";

#[post("/new-discord-message")]
async fn new_discord_message(
    req: HttpRequest,
    chat_server: web::Data<ChatServerHandle>,
    new_chat: Json<DiscordToClanChatMessage>,
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
    discord_http_client: Data<Http>,
    redis_client: Data<redis::Client>,
    new_chat: Json<Vec<ClanMessage>>,
    mongodb: Data<BotMongoDb>,
    celery: Data<Arc<Celery>>,
) -> actix_web::Result<String> {
    let possible_verification_code = req.headers().get("verification-code");
    if let None = possible_verification_code {
        let result = Err(MyError {
            message: "No verification code was set",
        });
        return result.map_err(|err: MyError| error::ErrorBadRequest(err.message));
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

    let mut clan_chat_queue: Vec<CreateEmbed> = vec![];
    let mut broadcast_queue: Vec<CreateEmbed> = vec![];
    let mut leagues_broadcast_queue: Vec<CreateEmbed> = vec![];

    for mut chat in new_chat.clone() {
        if chat.sender.clone() == "" && chat.clan_name.clone() == "" {
            continue;
        }

        if chat.is_league_world.is_some() {
            if chat.is_league_world.unwrap() {
                // info!("Broadcast from League World")
            }
        }
        //Checks to make sure the message has not already been process since multiple people could be submitting them
        let message_content_hash =
            hash_string(format!("{}{}", chat.message.clone(), chat.sender.clone()));

        let mut redis_connection = redis_client
            .get_connection()
            .expect("Could not connect to redis");

        let redis_key = format!("MessageHashes:{}", message_content_hash);
        match fetch_redis::<String>(&mut redis_connection, &redis_key).await {
            Ok(_) => {
                continue;
            }
            Err(_) => {
                write_to_cache_with_seconds(&mut redis_connection, &redis_key, true, 10).await
            }
        }

        //HACK leagues. is_league_world is not getting set as expected so we need to check the icon_id
        if let Some(icon_id) = chat.icon_id {
            //League icon id
            chat.is_league_world = Some(icon_id == 22);
        }

        //Sets the clan name to the guild. Bit of a hack but best way to get clan name
        if let None = registered_guild.clan_name {
            registered_guild.clan_name = Some(chat.clan_name.clone());
            mongodb.guilds.update_guild(registered_guild.clone()).await
        }

        if registered_guild.clan_name.clone().unwrap() != chat.clan_name {
            //TODO may remove. it happens a lot assuming from ppl moving clans
            // error!("Clan name does not match the clan name saved in the database");
            continue;
        }

        let right_now = serenity::model::timestamp::Timestamp::now();

        match registered_guild.clan_chat_channel {
            Some(_channel_id) => {
                let author_image = match chat.clan_name.clone() == chat.sender.clone() {
                    true => {
                        "https://oldschool.runescape.wiki/images/Your_Clan_icon.png".to_string()
                    }
                    false => get_wiki_clan_rank_image_url(chat.rank.clone()),
                };

                clan_chat_queue.push(
                    CreateEmbed::new()
                        .title("")
                        .author(CreateEmbedAuthor::new(chat.sender.clone()).icon_url(author_image))
                        //HACK
                        .description(chat.message.clone().replace(LEAGUES_ICON_TAG, ""))
                        .color(0x0000FF)
                        .timestamp(right_now),
                );
            }
            _ => {}
        }
        //Checks to see if it is a clan broadcast. Clan name and sender are the same if so
        if chat.sender != chat.clan_name {
            //Handles RL chat commands
            if chat.message.starts_with("!") {
                let _ = celery
                    .send_task(
                        trackscape_discord_shared::jobs::parse_rl_chat_command::parse_command::new(
                            chat.message.clone(),
                            chat.sender.clone(),
                            registered_guild.guild_id,
                        ),
                    )
                    .await;
            }

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
            continue;
        }

        //TODO may remove this since the handler does some loging for the website now
        if registered_guild.broadcast_channel.is_none()
            && registered_guild.clan_chat_channel.is_none()
        {
            //If there is not any broadcast_channels set just continue
            continue;
        }

        // let league_world = chat.is_league_world.unwrap_or(false);
        //HACK leagues. is_league_world is not getting set as expected so we need to check the broadcast for the icon tag
        let league_world = chat.message.starts_with(LEAGUES_ICON_TAG);
        if league_world {
            chat.message = chat.message.replace(LEAGUES_ICON_TAG, "");
            chat.is_league_world = Some(true);
        }

        let item_mapping_from_redis = get_item_mapping(&mut redis_connection).await;

        let quests_from_redis = get_quests_and_difficulties(&mut redis_connection).await;

        let cloned_celery = Arc::clone(&**celery);
        let celery_job_queue = Arc::new(CeleryJobQueue {
            celery: cloned_celery,
        });

        let handler = OSRSBroadcastHandler::new(
            chat.clone(),
            item_mapping_from_redis,
            quests_from_redis,
            registered_guild.clone(),
            league_world,
            mongodb.drop_logs.clone(),
            mongodb.clan_mate_collection_log_totals.clone(),
            mongodb.clan_mates.clone(),
            celery_job_queue,
        );
        let possible_broadcast = handler.extract_message().await;

        match possible_broadcast {
            None => {
                if league_world {
                    //This checks for leagues only broadcasts. Like new area, etc
                    let possible_leagues_message = handler.extract_leagues_message().await;
                    if let Some(leagues_message) = possible_leagues_message {
                        let mut broadcast_embed = CreateEmbed::new()
                            .title(leagues_message.title.clone())
                            .description(leagues_message.message.clone())
                            .color(0x0000FF)
                            .timestamp(right_now);
                        match leagues_message.icon_url {
                            None => {}
                            Some(icon_url) => {
                                broadcast_embed = broadcast_embed.image(icon_url);
                            }
                        }

                        //Only send if theres a leagues channel
                        if let Some(_channel_to_send_broadcast) =
                            registered_guild.leagues_broadcast_channel
                        {
                            leagues_broadcast_queue.push(broadcast_embed);
                        }
                    }
                }
            }
            Some(broadcast) => {
                let _ = mongodb
                    .broadcasts
                    .create_broadcast(registered_guild.guild_id, broadcast.clone())
                    .await;
                let mut broadcast_embed = CreateEmbed::new()
                    .title(broadcast.title.clone())
                    .description(broadcast.message.clone())
                    .color(0x0000FF)
                    .timestamp(right_now);
                match broadcast.icon_url {
                    None => {}
                    Some(icon_url) => {
                        broadcast_embed = broadcast_embed.image(icon_url);
                    }
                }

                match league_world {
                    true => {
                        if registered_guild.leagues_broadcast_channel.is_some() {
                            leagues_broadcast_queue.push(broadcast_embed);
                        }
                    }
                    false => {
                        if registered_guild.broadcast_channel.is_some() {
                            broadcast_queue.push(broadcast_embed);
                        }
                    }
                };
            }
        };
    }

    //Send all the messages
    if clan_chat_queue.len() > 0 {
        if let Some(channel_id) = registered_guild.clan_chat_channel {
            let result = ChannelId::new(channel_id)
                .send_message(
                    &*discord_http_client,
                    CreateMessage::new().embeds(clan_chat_queue),
                )
                .await;
            if let Err(_e) = result {
                // error!("Error sending clan chat: {:?}", e);
            }
        }
    }

    if broadcast_queue.len() > 0 {
        if let Some(channel_id) = registered_guild.broadcast_channel {
            let result = ChannelId::new(channel_id)
                .send_message(
                    &*discord_http_client,
                    CreateMessage::new().embeds(broadcast_queue),
                )
                .await;
            if let Err(_e) = result {
                // error!("Error sending broadcast: {:?}", e);
            }
        }
    }

    if leagues_broadcast_queue.len() > 0 {
        if let Some(channel_id) = registered_guild.leagues_broadcast_channel {
            let result = ChannelId::new(channel_id)
                .send_message(
                    &*discord_http_client,
                    CreateMessage::new().embeds(leagues_broadcast_queue),
                )
                .await;
            if let Err(_e) = result {
                // error!("Error sending leagues broadcast: {:?}", e);
            }
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
