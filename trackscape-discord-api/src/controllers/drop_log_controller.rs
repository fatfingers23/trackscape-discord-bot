use actix_web::{get, web, Error, HttpResponse, Scope};
use chrono::Utc;
use csv::Writer;
use dateparser::parse_with_timezone;
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

#[derive(serde::Serialize)]
struct DropRow<'a> {
    #[serde(rename = "RSN")]
    username: &'a str,
    #[serde(rename = "Item Name")]
    item_name: &'a str,
    #[serde(rename = "Quantity")]
    quantity: i64,
    #[serde(rename = "Price")]
    price: Option<i64>,
    #[serde(rename = "Date")]
    date: chrono::DateTime<chrono::Utc>,
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
                let start = bson::datetime::DateTime::from_chrono(parsed_start_date.unwrap());
                let end = bson::datetime::DateTime::from_chrono(parsed_end_date.unwrap());
                let possible_drop_logs = mongodb
                    .drop_logs
                    .get_drops_between_dates(registered_guild.guild_id.clone(), start, end)
                    .await;

                match possible_drop_logs {
                    Ok(drop_logs) => {
                        if drop_logs.len() == 0 {
                            return Ok(HttpResponse::Ok().body("RSN,Item Name,Quantity,Price,Date"));
                        }
                        let mut wtr = Writer::from_writer(vec![]);
                        for drop_log in drop_logs.clone() {
                            let drop_row = DropRow {
                                username: drop_log.drop_item.player_it_happened_to.as_str(),
                                item_name: drop_log.drop_item.item_name.as_str(),
                                quantity: drop_log.drop_item.item_quantity,
                                price: drop_log.drop_item.item_value,
                                date: drop_log.created_at.to_chrono(),
                            };
                            wtr.serialize(drop_row).unwrap();
                        }
                        let result_of_writer = wtr.into_inner();
                        match result_of_writer {
                            Ok(csv_bytes) => {
                                let csv = String::from_utf8(csv_bytes).unwrap();
                                return Ok(HttpResponse::Ok().body(csv));
                            }
                            Err(_) => {
                                return Ok(HttpResponse::BadRequest()
                                    .body("There was an issue rendering the drop logs to csv."));
                            }
                        }
                    }
                    Err(_) => {
                        return Ok(HttpResponse::BadRequest()
                            .body("There was an issue getting the drop logs."));
                    }
                };
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
    web::scope("/drops").service(get_drops)
}
