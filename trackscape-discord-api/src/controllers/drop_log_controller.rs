use actix_web::{get, web, Error, HttpResponse, Scope};
use chrono::{DateTime, Utc};
use log::info;
use num_format::Locale::te;
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::{BotMongoDb, DropLogs, RegisteredGuildModel};

#[derive(Deserialize, Serialize)]
struct ListDropsRequest {
    confirmation_code: String,
    //Example: 2024-03-24T20:50:00+01:00
    start_date: DateTime<Utc>,
    //Example: 2024-03-24T20:50:00+01:00
    end_date: DateTime<Utc>,
}

#[get("/list/{confirmation_code}/{start_date}/{end_date}")]
async fn get_drops(
    list_drop_request: web::Path<ListDropsRequest>,
    mongodb: web::Data<BotMongoDb>,
) -> Result<HttpResponse, Error> {
    let possible_registered_guild = mongodb
        .guilds
        .get_guild_by_code(list_drop_request.confirmation_code.clone())
        .await;

    match possible_registered_guild {
        Ok(guild) => match guild {
            Some(registered_guild) => {
                let start = bson::datetime::DateTime::from_chrono(list_drop_request.start_date);
                let end = bson::datetime::DateTime::from_chrono(list_drop_request.end_date);
                let _state = mongodb
                    .drop_logs
                    .get_drops_between_dates(registered_guild.guild_id.clone(), start, end)
                    .await;

                return Ok(HttpResponse::Ok().json(_state.unwrap()));
            }
            None => {
                return Ok(HttpResponse::Unauthorized().body("Invalid Confirmation Code"));
            }
        },
        Err(_) => {
            return Ok(HttpResponse::BadRequest().body("There was an issue getting the clan."));
        }
    }

    // Ok(HttpResponse::Ok().json(ListDropsRequest {
    //     confirmation_code: "".to_string()
    //     start_date: list_drop_request.start_date.clone(),
    //     end_date: list_drop_request.end_date.clone(),
    // }))
    Ok(HttpResponse::Ok().json("".to_string()))
}

pub fn drop_log_controller() -> Scope {
    let right_now = Utc::now();
    info!("right now: {}", right_now);

    web::scope("/drops").service(get_drops)
}
