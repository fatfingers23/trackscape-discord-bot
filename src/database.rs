use mongodb::Database;
use serde::{Deserialize, Serialize};
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;

pub struct BotMongoDb {
    db: mongodb::Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordServer {
    pub server_id: u64,
    //Channel to send broadcast messages
    pub broadcast_channel: Option<u64>,
    //Channel to send clan chats messages
    pub clan_chat_channel: Option<u64>,
    pub drop_price_threshold: Option<u64>,
    pub allowed_broadcast_types: Option<Vec<BroadcastType>>,
}

impl BotMongoDb {
    pub fn new(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn save_new_server(&self, server_id: u64) {
        let server = DiscordServer {
            server_id,
            broadcast_channel: None,
            clan_chat_channel: None,
            drop_price_threshold: None,
            allowed_broadcast_types: None,
        };
        let collection = self.db.collection("servers");
        collection.insert_one(server, None).await.expect("Failed to insert document for a new server.");
    }
    pub fn get_or_set_server(&self){

    }
}
