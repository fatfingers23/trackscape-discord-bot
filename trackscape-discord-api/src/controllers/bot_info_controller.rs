use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Scope};
use bot_info_dto::DiscordServerCount;
use dto::bot_info_dto;
use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};

use std::sync::atomic::AtomicI64;
use std::sync::Mutex;
use trackscape_discord_shared::dto;
use web::Json;

#[derive(Debug, Serialize, Deserialize)]
struct BotInfo {
    server_count: i64,
    connected_users: i64,
    total_chat_messages_for_today: Option<i64>,
}

#[get("/landing-page-info")]
async fn get_landing_page_info(
    connected_websockets: Data<Mutex<usize>>,
    connected_discord_servers: Data<AtomicI64>,
    redis_client: Data<redis::Client>,
) -> Result<HttpResponse, Error> {
    let discord_server_count = connected_discord_servers.load(std::sync::atomic::Ordering::SeqCst);

    let mut redis_connection = redis_client
        .get_connection()
        .expect("Failed to get redis connection");

    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d");
    let clan_chat_stats_key = format!("chat_stats:{}:clan_chat", today);

    let total_chat_messages_for_today: RedisResult<i64> = redis_connection.get(clan_chat_stats_key);
    if let Ok(total_chat_messages_for_today) = total_chat_messages_for_today {
        return Ok(HttpResponse::Ok().json(BotInfo {
            server_count: discord_server_count as i64,
            connected_users: connected_websockets.lock().unwrap().clone() as i64,
            total_chat_messages_for_today: Some(total_chat_messages_for_today),
        }));
    }

    Ok(HttpResponse::Ok().json(BotInfo {
        server_count: discord_server_count as i64,
        connected_users: connected_websockets.lock().unwrap().clone() as i64,
        total_chat_messages_for_today: None,
    }))
}

#[post("/set-server-count")]
async fn set_discord_server_count(
    req: HttpRequest,
    model: Json<DiscordServerCount>,
    connected_discord_servers: Data<AtomicI64>,
) -> Result<HttpResponse, Error> {
    let Ok(server_api_key) = std::env::var("MANAGEMENT_API_KEY") else {
        println!("MANAGEMENT_API_KEY isn't set! This should never error out as it is checked in startup.");
        return Ok(HttpResponse::InternalServerError().body("Internal server error :("));
    };

    let Some(request_api_key) = req.headers().get("api-key") else {
        return Ok(HttpResponse::Unauthorized().body("Missing API Key"));
    };

    let request_api_key = request_api_key
        .to_str()
        .inspect_err(|f| println!("Got an error while converting Api-Key header to a String: {f}"))
        .unwrap();

    if server_api_key != request_api_key {
        return Ok(HttpResponse::Unauthorized().body("Invalid API Key"));
    }

    connected_discord_servers.store(model.server_count, std::sync::atomic::Ordering::SeqCst);
    Ok(HttpResponse::new(StatusCode::NO_CONTENT))
}

pub fn info_controller() -> Scope {
    web::scope("/info")
        .service(get_landing_page_info)
        .service(set_discord_server_count)
}
