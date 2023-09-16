use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Scope};
use bot_info_dto::DiscordServerCount;
use dto::bot_info_dto;

use serde::{Deserialize, Serialize};

use shuttle_persist::PersistInstance;

use std::sync::atomic::AtomicI64;
use std::sync::Mutex;
use trackscape_discord_shared::dto;
use web::Json;

#[derive(Debug, Serialize, Deserialize)]
struct BotInfo {
    server_count: i64,
    connected_users: i64,
}

#[get("/landing-page-info")]
async fn get_landing_page_info(
    connected_websockets: Data<Mutex<usize>>,
    connected_discord_servers: Data<AtomicI64>,
) -> Result<HttpResponse, Error> {
    let discord_server_count = connected_discord_servers.load(std::sync::atomic::Ordering::SeqCst);

    Ok(HttpResponse::Ok().json(BotInfo {
        server_count: discord_server_count as i64,
        connected_users: connected_websockets.lock().unwrap().clone() as i64,
    }))
}

#[post("/set-server-count")]
async fn set_discord_server_count(
    req: HttpRequest,
    model: Json<DiscordServerCount>,
    connected_discord_servers: Data<AtomicI64>,
    persist: web::Data<PersistInstance>,
) -> Result<HttpResponse, Error> {
    let possible_api_key = persist.load::<String>("api-key");

    let correct_api_key = match possible_api_key {
        Ok(api_key) => api_key,
        Err(_) => {
            return Ok(HttpResponse::Unauthorized().body("Invalid API Key"));
        }
    };
    let request_api_key = match req.headers().get("api-key") {
        Some(api_key) => api_key.to_str().unwrap(),
        None => {
            return Ok(HttpResponse::Unauthorized().body("Invalid API Key"));
        }
    };
    if correct_api_key != request_api_key {
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
