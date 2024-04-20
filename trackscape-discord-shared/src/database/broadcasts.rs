use crate::database::BroadcastsDb;
use crate::osrs_broadcast_handler::BroadcastMessageToDiscord;
use mockall::predicate::*;
use mongodb::bson::{doc, DateTime};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BroadcastModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub guild_id: u64,
    pub broadcast: BroadcastMessageToDiscord,
    pub created_at: Option<DateTime>,
}

impl BroadcastsDb {
    pub const COLLECTION_NAME: &'static str = "broadcasts";
    pub fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn create_broadcast(
        &self,
        guild_id: u64,
        broadcast: BroadcastMessageToDiscord,
    ) -> Result<(), anyhow::Error> {
        let collection = self.db.collection(Self::COLLECTION_NAME);
        let model = BroadcastModel {
            id: bson::oid::ObjectId::new(),
            guild_id,
            broadcast,
            created_at: Some(DateTime::now()),
        };
        collection.insert_one(model, None).await?;
        Ok(())
    }
}

impl BroadcastsDb {
    pub fn new(mongodb: Database) -> Self {
        Self { db: mongodb }
    }
}
