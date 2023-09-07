use crate::cache::Cache;
use actix_web::{error, get, post, web, web::ServiceConfig, Scope};
use serenity::builder::CreateMessage;
use serenity::http::Http;
use serenity::json;
use serenity::json::Value;
use trackscape_discord_shared::database::{BotMongoDb, RegisteredGuild};
use trackscape_discord_shared::helpers::hash_string;
use trackscape_discord_shared::osrs_broadcast_extractor::osrs_broadcast_extractor::ClanMessage;

#[post("/new-message")]
async fn hello_world(
    discord_http_client: web::Data<Http>,
    cache: web::Data<Cache>,
    new_chat: web::Json<ClanMessage>,
    mongodb: web::Data<BotMongoDb>,
) -> actix_web::Result<String> {
    //TODO add middle ware that checks the header for the confirmation code
    let code = "123-456-789";

    //Checks to make sure the message has not already been process since multiple people could be submitting them
    let message_content_hash = hash_string(new_chat.message.clone());
    match cache.get_value(message_content_hash.clone()).await {
        Some(_) => return Ok("Message already processed".to_string()),
        None => {
            cache
                .set_value(message_content_hash.clone(), "true".to_string())
                .await;
        }
    }

    //checks to make sure the registered guild exists for the RuneScape clan
    let registered_guild_query = mongodb
        .get_guild_by_code_and_clan_name(code.to_string(), new_chat.author.clone())
        .await;

    let registered_guild_successful_query = if let Ok(registered_guild) = registered_guild_query {
        registered_guild
    } else {
        registered_guild_query.map_err(|err| error::ErrorInternalServerError(err))?
    };

    let registered_guild = if let Some(registered_guild) = registered_guild_successful_query {
        registered_guild
    } else {
        return Ok("No guild found".to_string());
    };

    match registered_guild.clan_chat_channel {
        Some(channel_id) => {
            let mut create_message = CreateMessage::default();
            create_message.content(new_chat.message.clone());
            let map = json::hashmap_to_json_map(create_message.0);
            discord_http_client
                .send_message(channel_id, &Value::from(map))
                .await
                .unwrap();
            Ok("Message sent".to_string())
        }
        None => return Ok("No channel set".to_string()),
    }
}

pub fn chat_controller() -> Scope {
    web::scope("/chat").service(hello_world)
}
