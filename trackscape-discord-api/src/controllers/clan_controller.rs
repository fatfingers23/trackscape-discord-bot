use actix_web::{get, web, Error, HttpResponse, Scope};
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::clan_mates::ClanMates;
use trackscape_discord_shared::database::BotMongoDb;
use web::Data;

#[derive(Deserialize, Serialize)]
struct ClanViewModel {
    id: String,
    name: String,
    registered_members: u64,
}

#[get("/list")]
async fn list_clans(mongodb: Data<BotMongoDb>) -> Result<HttpResponse, Error> {
    let result = mongodb.guilds.list_clans().await;

    match result {
        Ok(clans) => {
            let mut view_models: Vec<ClanViewModel> = Vec::new();
            for clan in clans {
                view_models.push(ClanViewModel {
                    id: clan.id.to_string(),
                    name: clan.clan_name.unwrap(),
                    registered_members: mongodb
                        .clan_mates
                        .get_clan_member_count(clan.guild_id)
                        .await
                        .unwrap(),
                });
            }
            // .map(async |clan| {
            //     return ClanViewModel {
            //         id: clan.id.to_string(),
            //         name: clan.clan_name.unwrap(),
            //         registered_members: mongodb
            //             .clan_mates
            //             .get_clan_member_count(clan.guild_id)
            //             .await
            //             .unwrap(),
            //     };
            // })
            // .collect();

            Ok(HttpResponse::Ok().json(view_models))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to list clans.")),
    }
}

pub fn clan_controller() -> Scope {
    web::scope("/clans").service(list_clans)
}
