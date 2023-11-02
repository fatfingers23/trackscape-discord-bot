use crate::database::DropLogsDb;
use crate::osrs_broadcast_extractor::osrs_broadcast_extractor::DropItemBroadcast;
use async_trait::async_trait;
use futures::TryStreamExt;
use mockall::predicate::*;
use mockall::*;
use mongodb::bson::{doc, DateTime};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DropLogModel {
    #[serde(rename = "_id")]
    id: bson::oid::ObjectId,
    pub guild_id: u64,
    pub drop_item: DropItemBroadcast,
    pub created_at: DateTime,
}

impl DropLogModel {
    pub const COLLECTION_NAME: &'static str = "drop_logs";

    pub fn new(drop_item: DropItemBroadcast, guild_id: u64) -> Self {
        Self {
            id: bson::oid::ObjectId::new(),
            guild_id,
            drop_item,
            created_at: DateTime::now(),
        }
    }
}

#[automock]
#[async_trait]
pub trait DropLogs {
    fn new_instance(mongodb: Database) -> Self;
    async fn new_drop_log(&self, drop_log: DropItemBroadcast, guild_id: u64);
    async fn get_drops_between_dates(
        &self,
        guild_id: u64,
        start_date: DateTime,
        end_date: DateTime,
    ) -> anyhow::Result<Vec<DropLogModel>>;
}

#[async_trait]
impl DropLogs for DropLogsDb {
    fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    async fn new_drop_log(&self, drop_broadcast: DropItemBroadcast, guild_id: u64) {
        let collection = self.db.collection(DropLogModel::COLLECTION_NAME);
        let new_drop_log = DropLogModel::new(drop_broadcast, guild_id);

        collection
            .insert_one(new_drop_log, None)
            .await
            .expect("Failed to insert document for a new drop log.");
    }

    async fn get_drops_between_dates(
        &self,
        guild_id: u64,
        start_date: DateTime,
        end_date: DateTime,
    ) -> anyhow::Result<Vec<DropLogModel>> {
        let collection = self
            .db
            .collection::<DropLogModel>(DropLogModel::COLLECTION_NAME);

        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
            "created_at": {
                "$gte": bson::to_bson(&start_date).unwrap(),
                "$lte": bson::to_bson(&end_date).unwrap()
            }
        };
        let result = collection.find(filter, None).await;
        return match result {
            Ok(possible_drops) => Ok(possible_drops.try_collect().await.unwrap()),
            Err(e) => Err(anyhow::Error::new(e)),
        };
    }
}
