use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::BroadcastType;
use mongodb::bson::doc;
use mongodb::Database;
use rand::Rng;
use regex::Error;
use serde::{Deserialize, Serialize};
use std::string::ToString;
use xxhash_rust::xxh3::Xxh3;

pub struct BotMongoDb {
    db: Database,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisteredGuild {
    pub guild_id: u64,
    //Channel to send broadcast messages
    pub clan_name: Option<String>,
    pub broadcast_channel: Option<u64>,
    //Channel to send clan chats messages
    pub clan_chat_channel: Option<u64>,
    pub drop_price_threshold: Option<u64>,
    pub allowed_broadcast_types: Option<Vec<BroadcastType>>,
    pub verification_code: String,
    pub hashed_verification_code: String,
    pub verified: bool,
}

impl RegisteredGuild {
    pub const COLLECTION_NAME: &'static str = "guilds";
    fn new(guild_id: u64) -> Self {
        let verification_code = Self::generate_code();
        let hashed_verification_code = Self::hash_code(verification_code.clone());
        Self {
            guild_id,
            clan_name: None,
            broadcast_channel: None,
            clan_chat_channel: None,
            drop_price_threshold: None,
            allowed_broadcast_types: None,
            verification_code,
            hashed_verification_code,
            verified: false,
        }
    }

    fn generate_code() -> String {
        let mut code = String::new();
        let mut rng = rand::thread_rng();

        for i in 0..9 {
            code.push_str(&rng.gen_range(0..10).to_string());

            if i == 2 || i == 5 {
                code.push('-');
            }
        }
        code
    }

    fn hash_code(code: String) -> String {
        let mut hasher = Xxh3::new();
        hasher.update(code.as_ref());
        hasher.digest().to_string()
    }
}

impl BotMongoDb {
    pub fn new(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn save_new_guild(&self, server_id: u64) {
        let server = RegisteredGuild::new(server_id);
        let collection = self.db.collection(RegisteredGuild::COLLECTION_NAME);
        collection
            .insert_one(server, None)
            .await
            .expect("Failed to insert document for a new server.");
    }
    pub async fn get_guild_by_code_and_clan_name(
        &self,
        code: String,
        clan_name: String,
    ) -> Result<Option<RegisteredGuild>, Error> {
        let collection = self
            .db
            .collection::<RegisteredGuild>(RegisteredGuild::COLLECTION_NAME);
        let hashed_code = RegisteredGuild::hash_code(code);
        let filter = doc! {"hashed_verification_code": hashed_code, "clan_name": clan_name};
        Ok(collection
            .find_one(filter, None)
            .await
            .expect("Failed to find document for the Discord Server."))
    }

    pub async fn get_by_guild_id(&self, id: u64) -> Result<Option<RegisteredGuild>, Error> {
        let collection = self
            .db
            .collection::<RegisteredGuild>(RegisteredGuild::COLLECTION_NAME);
        let filter = doc! { "guild_id": id.to_string()};
        Ok(collection
            .find_one(filter, None)
            .await
            .expect("Failed to find document for the Discord Server."))
    }
}
