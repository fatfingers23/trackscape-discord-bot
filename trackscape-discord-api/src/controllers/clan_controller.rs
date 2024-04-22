use actix_web::{get, web, Error, HttpResponse, Scope};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use trackscape_discord_shared::database::clan_mate_collection_log_totals::ClanMateCollectionLogTotals;
use trackscape_discord_shared::database::clan_mates::{ClanMateModel, ClanMates};
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

#[get("/list")]
async fn list_clans(mongodb: Data<BotMongoDb>) -> Result<HttpResponse, Error> {
    let result = mongodb.guilds.list_clans().await;

    match result {
        Ok(clans) => {
            let mut view_models: Vec<ClanViewModel> = Vec::new();
            for clan in clans {
                if clan.clan_name.is_none() {
                    continue;
                }
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

            Ok(HttpResponse::Ok().json(view_models))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to list clans.")),
    }
}

#[get("/{id}/detail")]
async fn detail(
    mongodb: Data<BotMongoDb>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    info!("{:?}", path);
    let id = path.into_inner().0;
    let possible_parsed_id = bson::oid::ObjectId::from_str(id.as_str());
    let id = match possible_parsed_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };

    let registered_guild_query = mongodb.guilds.get_by_id(id).await;
    match registered_guild_query {
        Ok(possible_registered_guild) => match possible_registered_guild {
            None => {
                return Ok(HttpResponse::NotFound().body("Clan not found."));
            }
            Some(registered_guild) => {
                //return clan details

                let detail = ClanDetail {
                    id: registered_guild.id.to_string(),
                    name: registered_guild.clan_name.unwrap(),
                    discord_guild_id: registered_guild.guild_id.to_string(),
                    registered_members: mongodb
                        .clan_mates
                        .get_clan_member_count(registered_guild.guild_id)
                        .await
                        .unwrap(),
                    members: mongodb
                        .clan_mates
                        .get_clan_mates_by_guild_id(registered_guild.guild_id)
                        .await
                        .unwrap(),
                };
                return Ok(HttpResponse::Ok().json(detail));
            }
        },
        Err(err) => {
            error!("Failed to get clan by id: {}", err);
            return Ok(HttpResponse::BadRequest().body("There was an issue with the request"));
        }
    }
}

#[get("/{id}/collection-log")]
async fn collection_log(
    mongodb: Data<BotMongoDb>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner().0;
    let possible_parsed_id = bson::oid::ObjectId::from_str(id.as_str());
    let id = match possible_parsed_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };

    let registered_guild_query = mongodb.guilds.get_by_id(id).await;
    match registered_guild_query {
        Ok(possible_registered_guild) => match possible_registered_guild {
            None => {
                return Ok(HttpResponse::NotFound().body("Clan not found."));
            }
            Some(registered_guild) => {
                //return clan details
                let result = mongodb
                    .clan_mate_collection_log_totals
                    .get_guild_totals(registered_guild.guild_id)
                    .await;
                match result {
                    Ok(totals) => {
                        return Ok(HttpResponse::Ok().json(totals));
                    }
                    Err(err) => {
                        error!("Failed to get clan by id: {}", err);
                        return Ok(
                            HttpResponse::BadRequest().body("There was an issue with the request")
                        );
                    }
                }
            }
        },
        Err(err) => {
            error!("Failed to get clan by id: {}", err);
            return Ok(HttpResponse::BadRequest().body("There was an issue with the request"));
        }
    }
}

#[derive(Deserialize)]
struct BroadcastRequest {
    id: String,
    limit: i64,
}

#[get("/{id}/broadcasts/{limit}")]
async fn broadcasts(
    mongodb: Data<BotMongoDb>,
    path: web::Path<BroadcastRequest>,
) -> Result<HttpResponse, Error> {
    let possible_parsed_id = bson::oid::ObjectId::from_str(path.id.as_str());
    let id = match possible_parsed_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };
    let registered_guild_query = mongodb.guilds.get_by_id(id).await;
    match registered_guild_query {
        Ok(possible_guild) => match possible_guild {
            Some(guild) => {
                let limit_to_use = if path.limit > 100 { 100 } else { path.limit };
                let broadcasts = mongodb
                    .broadcasts
                    .get_latest_broadcasts(guild.guild_id, limit_to_use)
                    .await;
                match broadcasts {
                    Ok(broadcasts) => Ok(HttpResponse::Ok().json(broadcasts)),
                    Err(err) => {
                        error!("Failed to get broadcasts: {}", err);
                        Ok(HttpResponse::BadRequest().body("There was an issue with the request"))
                    }
                }
            }
            None => Ok(HttpResponse::BadRequest().body("There is not a clan with that id")),
        },
        Err(err) => {
            error!("Failed to get clan by id: {}", err);
            Ok(HttpResponse::BadRequest().body("There was an issue with the request"))
        }
    }
}

#[get("/{guild_id}/{activity_id}/personal-bests")]
async fn personal_bests(
    mongodb: Data<BotMongoDb>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let get_variables = path.into_inner();
    let guild_id = get_variables.0;
    let possible_parsed_guild_id = bson::oid::ObjectId::from_str(guild_id.as_str());
    let guild_id = match possible_parsed_guild_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };

    let activity_id = get_variables.1;
    let possible_parsed_activity_id = bson::oid::ObjectId::from_str(activity_id.as_str());
    let activity_id = match possible_parsed_activity_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };

    let registered_guild_query = mongodb.guilds.get_by_id(guild_id).await;
    match registered_guild_query {
        Ok(possible_registered_guild) => match possible_registered_guild {
            None => {
                return Ok(HttpResponse::NotFound().body("Clan not found."));
            }
            Some(registered_guild) => {
                let result = mongodb
                    .pb_records
                    .get_pb_records_leaderboard(activity_id, registered_guild.guild_id)
                    .await;
                match result {
                    Ok(records) => Ok(HttpResponse::Ok().json(records)),
                    Err(err) => {
                        error!("Failed to get personal bests: {}", err);
                        Ok(HttpResponse::BadRequest().body("There was an issue with the request"))
                    }
                }
            }
        },
        Err(err) => {
            error!("Failed to get clan by id: {}", err);
            return Ok(HttpResponse::BadRequest().body("There was an issue with the request"));
        }
    }
}

pub fn clan_controller() -> Scope {
    web::scope("/clans")
        .service(list_clans)
        .service(detail)
        .service(collection_log)
        .service(broadcasts)
        .service(personal_bests)
}
