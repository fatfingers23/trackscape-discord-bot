use crate::database::ClanMateCollectionLogTotalsDb;
use anyhow::Error;
use async_trait::async_trait;
use log::info;
use mockall::automock;
use mongodb::bson::{doc, DateTime};
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanMateCollectionLogTotalModel {
    pub guild_id: u64,
    pub player_id: bson::oid::ObjectId,
    pub total: i64,
    pub created_at: DateTime,
}

impl ClanMateCollectionLogTotalModel {
    pub const COLLECTION_NAME: &'static str = "clan_mate_collection_log_totals";

    pub fn new(guild_id: u64, player_id: bson::oid::ObjectId, total: i64) -> Self {
        Self {
            guild_id,
            player_id,
            total,
            created_at: DateTime::now(),
        }
    }
}

#[automock]
#[async_trait]
pub trait ClanMateCollectionLogTotals {
    fn new_instance(mongodb: Database) -> Self;

    async fn update_or_create(
        &self,
        guild_id: u64,
        player_id: bson::oid::ObjectId,
        total: i64,
    ) -> Result<(), anyhow::Error>;
}

#[async_trait]
impl ClanMateCollectionLogTotals for ClanMateCollectionLogTotalsDb {
    fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    async fn update_or_create(
        &self,
        guild_id: u64,
        player_id: bson::oid::ObjectId,
        total: i64,
    ) -> Result<(), Error> {
        let collection = self.db.collection::<ClanMateCollectionLogTotalModel>(
            ClanMateCollectionLogTotalModel::COLLECTION_NAME,
        );

        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
            "player_id": player_id.clone(),
        };

        let update_result = collection
            .update_one(
                filter,
                doc! {
                    "$set": {
                        "total": bson::to_bson(&total).unwrap()
                    }
                },
                None,
            )
            .await?;
        info!("Update result: {:?}", update_result);
        if update_result.matched_count > 0 {
            return Ok(());
        }
        let new_total = ClanMateCollectionLogTotalModel::new(guild_id, player_id, total);
        let _ = collection
            .insert_one(new_total, None)
            .await
            .expect("Error inserting new collection log total.");
        Ok(())
    }
}
