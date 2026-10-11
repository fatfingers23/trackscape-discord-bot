extern crate dotenv;

mod controllers;
mod handler;
mod websocket_server;

use crate::controllers::bot_info_controller::info_controller;
use crate::controllers::chat_controller::chat_controller;
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{guard, web, App, Error, HttpServer};
use controllers::application_data_controller::application_data_controller;
use dotenv::dotenv;
use env_logger::Env;
use serenity::http::HttpBuilder;
use std::env;
use std::sync::atomic::AtomicI64;
use std::sync::Mutex;
use tokio::spawn;
use trackscape_discord_shared::database::{BotMongoDb, MongoDb};
use trackscape_discord_shared::ge_api::ge_api::get_item_mapping;
use trackscape_discord_shared::jobs::job_helpers::get_redis_client;
use uuid::Uuid;

pub use self::websocket_server::{ChatServer, ChatServerHandle};
use crate::controllers::clan_controller::clan_controller;
use crate::controllers::drop_log_controller::drop_log_controller;
use actix_files::{Files, NamedFile};
use log::{error, info};
use trackscape_discord_shared::jobs::get_celery_caller;
use trackscape_discord_shared::wiki_api::wiki_api::get_quests_and_difficulties;
use trackscape_discord_shared::wiki_api::wiki_api::get_clogs_and_percentages;

/// Connection ID.
pub type ConnId = Uuid;

/// Used to create a chat room for a cla
pub type VerificationCode = String;

/// Message sent to a clan/client.
pub type Msg = String;

async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("./trackscape-discord-api/ui/index.html")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let _ = env::var("MANAGEMENT_API_KEY").expect("MANAGEMENT_API_KEY not set!");
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    let mongodb_url = env::var("MONGO_DB_URL").expect("MONGO_DB_URL not set!");
    let production_env = env::var("PRODUCTION");
    let mut _is_production = false;
    match production_env {
        Ok(env) => _is_production = env == "true",
        Err(_) => {}
    }

    let db = BotMongoDb::new_db_instance(mongodb_url).await;
    let redis_client = get_redis_client();
    let mut redis_conn = redis_client
        .get_connection()
        .expect("Could not connect to redis");

    info!("Loading startup data from the web");
    let ge_mapping_request = get_item_mapping(&mut redis_conn).await;
    match ge_mapping_request {
        Ok(_) => {
            info!("GE mapping was out of date, updating cache");
        }
        Err(error) => {
            info!("Error getting ge mapping: {}", error)
        }
    }

    let possible_quests = get_quests_and_difficulties(&mut redis_conn).await;
    match possible_quests {
        Ok(_) => {
            info!("Quest mapping was out of date, updating cache");
        }
        Err(e) => {
            error!("Error getting quests: {}", e)
        }
    }

    let possible_clogs = get_clogs_and_percentages(&mut redis_conn).await;
    match possible_clogs {
        Ok(_) => {
            info!("Clog mapping was out of date, updating cache");
        }
        Err(e) => {
            error!("Error getting clogs: {}", e)
        }
    }

    #[allow(clippy::mutex_atomic)] // it's intentional.
    let connected_websockets_counter = Data::new(Mutex::new(0usize));
    let connected_discord_servers = Data::new(AtomicI64::new(0));

    let (chat_server, server_tx) = ChatServer::new(connected_websockets_counter.clone());

    let _ = spawn(chat_server.run());
    let celery = get_celery_caller().await;

    // Wrap shared state in `Data` here so the per-worker config closure below can be
    // `Fn + Send + Clone + 'static`, which `HttpServer::new` requires.
    let db = Data::new(db);
    let discord_http = Data::new(HttpBuilder::new(discord_token).build());
    let celery = Data::new(celery);
    let redis_client = Data::new(redis_client);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .expect("PORT is not a number");

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api")
                    .service(chat_controller())
                    .service(info_controller())
                    .service(drop_log_controller())
                    .service(clan_controller())
                    .service(application_data_controller())
                    .wrap(
                        Cors::default()
                            .allow_any_origin()
                            .allow_any_method()
                            .allow_any_header(),
                    ),
            )
            .service(Files::new("/", "./trackscape-discord-api/ui/").index_file("index.html"))
            .app_data(web::Data::new(server_tx.clone()))
            .app_data(connected_websockets_counter.clone())
            .app_data(connected_discord_servers.clone())
            .app_data(discord_http.clone())
            .app_data(db.clone())
            .app_data(celery.clone())
            .app_data(redis_client.clone())
            .default_service(web::route().guard(guard::Not(guard::Get())).to(index))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
