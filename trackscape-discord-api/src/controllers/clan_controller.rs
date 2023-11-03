use actix_web::{get, web, Error, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::BotMongoDb;
use web::Data;

#[derive(Deserialize, Serialize)]
struct ClanViewModel {
    id: String,
    name: String,
}

#[get("/list")]
async fn list_clans(mongodb: Data<BotMongoDb>) -> Result<HttpResponse, Error> {
    let result = mongodb.guilds.list_clans().await;

    match result {
        Ok(clans) => {
            let view_models: Vec<ClanViewModel> = clans
                .into_iter()
                .map(|clan| ClanViewModel {
                    id: clan.id.to_string(),
                    name: clan.clan_name.unwrap(),
                })
                .collect();
            Ok(HttpResponse::Ok().json(view_models))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to list clans.")),
    }
}

pub fn clan_controller() -> Scope {
    web::scope("/clans").service(list_clans)
}
