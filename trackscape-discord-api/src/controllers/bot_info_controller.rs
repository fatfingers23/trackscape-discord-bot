use crate::cache::Cache;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{get, post, web, Error, HttpRequest, HttpResponse, Responder, Scope};
use bot_info_dto::DiscordServerCount;
use dto::bot_info_dto;
use serde::{Deserialize, Serialize};
use serenity::http::{CacheHttp, Http};
use std::sync::atomic::{AtomicI64, AtomicUsize};
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
    connected_discord_servers: Data<AtomicUsize>,
) -> Result<HttpResponse, Error> {
    let discord_server_count = connected_discord_servers.load(std::sync::atomic::Ordering::SeqCst);

    Ok(HttpResponse::Ok().json(BotInfo {
        server_count: discord_server_count as i64,
        connected_users: connected_websockets.lock().unwrap().clone() as i64,
    }))
}

#[post("/set-server-count")]
async fn set_discord_server_count(
    model: Json<DiscordServerCount>,
    connected_discord_servers: Data<AtomicI64>,
) -> Result<HttpResponse, Error> {
    connected_discord_servers.store(model.server_count, std::sync::atomic::Ordering::SeqCst);
    Ok(HttpResponse::new(StatusCode::NO_CONTENT))
}

pub fn info_controller() -> Scope {
    web::scope("/info").service(get_landing_page_info)
}
