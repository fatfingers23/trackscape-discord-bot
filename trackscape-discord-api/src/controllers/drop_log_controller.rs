use actix_web::{get, web, Error, HttpResponse, Scope};
use chrono::Utc;
use dateparser::parse_with_timezone;
use log::info;
use serde::{Deserialize, Serialize};
use trackscape_discord_shared::database::{BotMongoDb, DropLogs};

#[derive(Deserialize, Serialize)]
struct ListDropsRequest {
    confirmation_code: String,
    //Example: 2024-03-24T20:50:00+01:00
    start_date: String,
    //Example: 2024-03-24T20:50:00+01:00
    end_date: String,
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
                let parsed_start_date = parse_with_timezone(&list_drop_request.start_date, &Utc);
                let parsed_end_date = parse_with_timezone(&list_drop_request.end_date, &Utc);
                if parsed_start_date.is_err() {
                    return Ok(HttpResponse::BadRequest().body("Invalid Start Date"));
                }
                if parsed_end_date.is_err() {
                    return Ok(HttpResponse::BadRequest().body("Invalid End Date"));
                }
                println!("parsed_start_date: {:?}", parsed_start_date);
                let start = bson::datetime::DateTime::from_chrono(parsed_start_date.unwrap());
                let end = bson::datetime::DateTime::from_chrono(parsed_end_date.unwrap());
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
}

pub fn drop_log_controller() -> Scope {
    let right_now = Utc::now();
    info!("right now: {}", right_now);

    web::scope("/drops").service(get_drops)
}
