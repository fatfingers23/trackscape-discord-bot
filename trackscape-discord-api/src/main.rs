extern crate dotenv;
mod cache;
mod controllers;
mod handler;
mod websocket_server;

use crate::cache::Cache;
use crate::controllers::bot_info_controller::info_controller;
use crate::controllers::chat_controller::chat_controller;
use actix_web::web::Data;
use actix_web::{web, web::ServiceConfig};
use dotenv::dotenv;
use serenity::http::HttpBuilder;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_persist::PersistInstance;
use shuttle_runtime::tracing::info;
use std::env;
use std::sync::atomic::{AtomicI64, AtomicUsize};
use std::sync::Mutex;
use std::time::Duration;
use tokio::spawn;
use trackscape_discord_shared::database::BotMongoDb;
use trackscape_discord_shared::ge_api::ge_api::{get_item_mapping, GeItemMapping};
use uuid::Uuid;

pub use self::websocket_server::{ChatServer, ChatServerHandle};

/// Connection ID.
pub type ConnId = Uuid;

/// Used to create a chat room for a cla
pub type VerificationCode = String;

/// Message sent to a clan/client.
pub type Msg = String;

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    dotenv().ok();

    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set!");
    let mongodb_url = env::var("MONGO_DB_URL").expect("MONGO_DB_URL not set!");
    let db = BotMongoDb::new_db(mongodb_url).await;

    let ge_mapping_request = get_item_mapping().await;
    match ge_mapping_request {
        Ok(ge_mapping) => {
            let _state = persist
                .save::<GeItemMapping>("mapping", ge_mapping.clone())
                .map_err(|e| info!("Saving Item Mapping Error: {e}"));
        }
        Err(error) => {
            info!("Error getting ge mapping: {}", error)
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

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(
            web::scope("/api")
                .service(chat_controller())
                .service(info_controller()),
        )
        .app_data(web::Data::new(server_tx.clone()))
        .app_data(connected_websockets_counter)
        .app_data(connected_discord_servers)
        .app_data(web::Data::new(cache_clone))
        .app_data(web::Data::new(HttpBuilder::new(discord_token).build()))
        .app_data(web::Data::new(db))
        .app_data(web::Data::new(persist.clone()));
    };
    Ok(config.into())
}
