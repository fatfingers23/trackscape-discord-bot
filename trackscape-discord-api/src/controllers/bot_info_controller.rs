use actix_web::{get, web, Error, HttpRequest, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use serenity::http::{CacheHttp, Http};

#[derive(Debug, Serialize, Deserialize)]
struct BotInfo {
    server_count: i64,
    connected_users: i64,
}

#[get("/landing-page-info")]
async fn get_landing_page_info(
    req: HttpRequest,
    discord_http_client: web::Data<Http>,
) -> Result<HttpResponse, Error> {
    //Get count on bot start. send web request to a data of it in actix

    Ok(HttpResponse::Ok().json(BotInfo {
        server_count: 0,
        connected_users: 0,
    }))
}

pub fn info_controller(cfg: &mut actix_web::web::ServiceConfig) -> Scope {
    web::scope("/info").service(get_landing_page_info)
}
