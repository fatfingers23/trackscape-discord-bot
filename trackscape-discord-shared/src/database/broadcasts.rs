use crate::database::BroadcastsDb;
use crate::osrs_broadcast_handler::BroadcastMessageToDiscord;
use bson::DateTime;
use futures::TryStreamExt;
use mockall::predicate::*;
use mongodb::bson::doc;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BroadcastModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub guild_id: u64,
    pub broadcast: BroadcastMessageToDiscord,
    #[serde(serialize_with = "bson::serde_helpers::serialize_bson_datetime_as_rfc3339_string")]
    #[serde(
        deserialize_with = "bson::serde_helpers::deserialize_bson_datetime_from_rfc3339_string"
    )]
    pub created_at: DateTime,
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
            created_at: DateTime::now(),
        };
        collection.insert_one(model, None).await?;
        Ok(())
    }

    pub async fn get_latest_broadcasts(
        &self,
        guild_id: u64,
        limit: i64,
    ) -> Result<Vec<BroadcastModel>, anyhow::Error> {
        let collection = self.db.collection(Self::COLLECTION_NAME);
        let filter = doc! { "guild_id": bson::to_bson(&guild_id).unwrap() };
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .build();
        let cursor = collection.find(filter, options).await?;
        let broadcasts: Vec<BroadcastModel> = cursor.try_collect().await?;
        Ok(broadcasts)
    }
}

impl BroadcastsDb {
    pub fn new(mongodb: Database) -> Self {
        Self { db: mongodb }
    }
}
