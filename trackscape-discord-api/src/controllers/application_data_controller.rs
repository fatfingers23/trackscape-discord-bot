use actix_web::{get, web, Error, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::clan_mates::ClanMateModel;
use trackscape_discord_shared::database::BotMongoDb;
use web::Data;

#[derive(Deserialize, Serialize)]
struct ClanViewModel {
    id: String,
    name: String,
    registered_members: u64,
}

#[derive(Deserialize, Serialize)]
struct ClanDetail {
    id: String,
    name: String,
    discord_guild_id: String,
    registered_members: u64,
    members: Vec<ClanMateModel>,
}

#[get("/activities")]
async fn get_pb_activities(mongodb: Data<BotMongoDb>) -> Result<HttpResponse, Error> {
    let result = mongodb.pb_activities.get_activities().await;

    match result {
        Ok(activities) => Ok(HttpResponse::Ok().json(activities)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to list activities.")),
    }
}

pub fn application_data_controller() -> Scope {
    web::scope("/resources").service(get_pb_activities)
}
