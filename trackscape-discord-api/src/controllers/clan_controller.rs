use actix_web::{Error, HttpResponse, Scope, get, web};
use log::{error, info};
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::BotDb;
use trackscape_discord_shared::database::clan_mate_collection_log_totals::ClanMateCollectionLogTotals;
use trackscape_discord_shared::database::clan_mates::{ClanMateModel, ClanMates};
use uuid::Uuid;
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

#[derive(Deserialize)]
struct ListClansQuery {
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
struct ListClansResponse {
    clans: Vec<ClanViewModel>,
    total: i64,
}

#[get("/list")]
async fn list_clans(
    db: Data<BotDb>,
    query: web::Query<ListClansQuery>,
) -> Result<HttpResponse, Error> {
    let search = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|search| !search.is_empty());
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let clans = db.guilds.list_clans_paginated(search, limit, offset).await;
    let total = db.guilds.count_clans(search).await;

    match (clans, total) {
        (Ok(clans), Ok(total)) => {
            let view_models = clans
                .into_iter()
                .map(|clan| ClanViewModel {
                    id: clan.id.to_string(),
                    name: clan.clan_name,
                    registered_members: clan.registered_members as u64,
                })
                .collect();
            Ok(HttpResponse::Ok().json(ListClansResponse {
                clans: view_models,
                total,
            }))
        }
        (clans, total) => {
            error!(
                "Failed to list clans: clans={:?} total={:?}",
                clans.err(),
                total.err()
            );
            Ok(HttpResponse::InternalServerError().body("Failed to list clans."))
        }
    }
}

#[get("/{id}/detail")]
async fn detail(mongodb: Data<BotDb>, path: web::Path<(String,)>) -> Result<HttpResponse, Error> {
    info!("{:?}", path);
    let id = path.into_inner().0;
    let possible_parsed_id = Uuid::parse_str(id.as_str());
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
                        .get_active_clan_mates_by_guild_id(registered_guild.guild_id)
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
    mongodb: Data<BotDb>,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner().0;
    let possible_parsed_id = Uuid::parse_str(id.as_str());
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
    mongodb: Data<BotDb>,
    path: web::Path<BroadcastRequest>,
) -> Result<HttpResponse, Error> {
    let possible_parsed_id = Uuid::parse_str(path.id.as_str());
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
    mongodb: Data<BotDb>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
    let get_variables = path.into_inner();
    let guild_id = get_variables.0;
    let possible_parsed_guild_id = Uuid::parse_str(guild_id.as_str());
    let guild_id = match possible_parsed_guild_id {
        Ok(parsed_id) => parsed_id,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("Invalid id format."));
        }
    };

    let activity_id = get_variables.1;
    let possible_parsed_activity_id = Uuid::parse_str(activity_id.as_str());
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
