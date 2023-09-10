mod cache;
mod controllers;
mod handler;
mod websocket_server;

use crate::cache::Cache;
use crate::controllers::chat_controller::chat_controller;
use actix_web::{web, web::ServiceConfig};
use anyhow::anyhow;
use mongodb::Database;
use serenity::http::HttpBuilder;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_persist::PersistInstance;
use shuttle_runtime::tracing::info;
use shuttle_secrets::SecretStore;
use std::time::Duration;
use tokio::spawn;
use trackscape_discord_shared::database::BotMongoDb;
use trackscape_discord_shared::ge_api::ge_api::{get_item_mapping, GeItemMapping};

pub use self::websocket_server::{ChatServer, ChatServerHandle};

/// Connection ID.
pub type ConnId = usize;

/// Used to create a chat room for a clan
pub type VerificationCode = String;

/// Message sent to a clan/client.
pub type Msg = String;

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::MongoDb(local_uri = "{secrets.MONGO_DB_URL}")] db: Database,
    #[shuttle_persist::Persist] persist: PersistInstance,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let discord_token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

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

    let (chat_server, server_tx) = ChatServer::new();

    let _ = spawn(chat_server.run());

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(web::scope("/api").service(chat_controller()))
            .app_data(web::Data::new(server_tx.clone()))
            // .app_data(web::Data::new(Arc::new(chat_server.clone())))
            .app_data(web::Data::new(cache_clone))
            .app_data(web::Data::new(HttpBuilder::new(discord_token).build()))
            .app_data(web::Data::new(BotMongoDb::new(db)))
            .app_data(web::Data::new(persist.clone()));
    };
    Ok(config.into())
}
