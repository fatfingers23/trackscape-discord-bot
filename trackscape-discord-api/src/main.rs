extern crate dotenv;

mod cache;
mod controllers;
mod handler;
mod websocket_server;

use crate::cache::Cache;
use crate::controllers::bot_info_controller::info_controller;
use crate::controllers::chat_controller::chat_controller;
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{guard, web, web::ServiceConfig, Error};
use controllers::application_data_controller::application_data_controller;
use dotenv::dotenv;
use serenity::http::HttpBuilder;
use shuttle_actix_web::ShuttleActixWeb;
use std::env;
use std::sync::atomic::AtomicI64;
use std::sync::Mutex;
use std::time::Duration;
use tokio::spawn;
use trackscape_discord_shared::database::{BotMongoDb, MongoDb};
use trackscape_discord_shared::ge_api::ge_api::{get_item_mapping, GeItemMapping};
use trackscape_discord_shared::jobs::job_helpers::get_redis_client;
use trackscape_discord_shared::jobs::redis_helpers::write_to_cache_with_seconds;

use uuid::Uuid;

pub use self::websocket_server::{ChatServer, ChatServerHandle};
use crate::controllers::clan_controller::clan_controller;
use crate::controllers::drop_log_controller::drop_log_controller;
use actix_files::{Files, NamedFile};
use log::{error, info};
use trackscape_discord_shared::jobs::get_celery_caller;
use trackscape_discord_shared::wiki_api::wiki_api::{get_quests_and_difficulties, WikiQuest};

/// Connection ID.
pub type ConnId = Uuid;

/// Used to create a chat room for a cla
pub type VerificationCode = String;

/// Message sent to a clan/client.
pub type Msg = String;

async fn index() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("./trackscape-discord-api/ui/index.html")?)
}

#[shuttle_runtime::main]
async fn actix_web() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    let _ = env::var("MANAGEMENT_API_KEY").expect("MANAGEMENT_API_KEY not set!");
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    let mongodb_url = env::var("MONGO_DB_URL").expect("MONGO_DB_URL not set!");
    let production_env = env::var("PRODUCTION");
    let mut is_production = false;
    match production_env {
        Ok(env) => is_production = env == "true",
        Err(_) => {}
    }

    let db = BotMongoDb::new_db_instance(mongodb_url).await;
    let mut redis_conn = get_redis_client().unwrap();

    if is_production {
        info!("Loading startup data from the web");
        let ge_mapping_request = get_item_mapping().await;
        match ge_mapping_request {
            Ok(ge_mapping) => {
                write_to_cache_with_seconds(&mut redis_conn, "mapping", ge_mapping, 604800).await;
            }
            Err(error) => {
                info!("Error getting ge mapping: {}", error)
            }
        }

        let possible_quests = get_quests_and_difficulties().await;
        match possible_quests {
            Ok(quests) => {
                write_to_cache_with_seconds(&mut redis_conn, "quests", quests, 604800).await;
            }
            Err(e) => {
                error!("Error getting quests: {}", e)
            }
        }
    }
    let mut cache = Cache::new(Duration::from_secs(10));
    let cache_clone = cache.clone();
    spawn(async move {
        cache.clean_expired().await;
    });

    #[allow(clippy::mutex_atomic)] // it's intentional.
    let connected_websockets_counter = Data::new(Mutex::new(0usize));
    let connected_discord_servers = Data::new(AtomicI64::new(0));

    let (chat_server, server_tx) = ChatServer::new(connected_websockets_counter.clone());

    let _ = spawn(chat_server.run());
    let celery = get_celery_caller().await;
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
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
        .app_data(connected_websockets_counter)
        .app_data(connected_discord_servers)
        .app_data(web::Data::new(cache_clone))
        .app_data(web::Data::new(HttpBuilder::new(discord_token).build()))
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(celery.clone()))
        .default_service(web::route().guard(guard::Not(guard::Get())).to(index));
    };

    Ok(config.into())
}
