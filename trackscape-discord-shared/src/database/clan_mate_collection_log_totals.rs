use crate::database::ClanMateCollectionLogTotalsDb;
use anyhow::Error;
use async_trait::async_trait;
use futures::TryStreamExt;
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
pub const COLLECTION_LOG_COLLECTION_NAME: &'static str = "clan_mate_collection_log_totals";

impl ClanMateCollectionLogTotalModel {
    pub fn new(guild_id: u64, player_id: bson::oid::ObjectId, total: i64) -> Self {
        Self {
            guild_id,
            player_id,
            total,
            created_at: DateTime::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClanMateCollectionLogTotalsView {
    pub player_name: String,
    pub total: i64,
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

    async fn get_guild_totals(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateCollectionLogTotalsView>, anyhow::Error>;
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
        let collection = self
            .db
            .collection::<ClanMateCollectionLogTotalModel>(COLLECTION_LOG_COLLECTION_NAME);

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

    async fn get_guild_totals(
        &self,
        guild_id: u64,
    ) -> Result<Vec<ClanMateCollectionLogTotalsView>, anyhow::Error> {
        let collection = self
            .db
            .collection::<ClanMateCollectionLogTotalModel>(COLLECTION_LOG_COLLECTION_NAME);

        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
        };

        let mut cursor = collection
            .aggregate(
                vec![
                    doc! {
                        "$match": filter
                    },
                    doc! {
                        "$lookup": {
                            "from": "clan_mates",
                            "localField": "player_id",
                            "foreignField": "_id",
                            "as": "clan_mate"
                        }
                    },
                    doc! {
                        "$unwind": "$clan_mate"
                    },
                    doc! {
                        "$project": {
                            "player_name": "$clan_mate.player_name",
                            "total": 1
                        }
                    },
                    doc! {
                        "$sort": {
                            "total": -1
                        }
                    },
                ],
                None,
            )
            .await?;

        let mut results: Vec<ClanMateCollectionLogTotalsView> = Vec::new();

        // Iterate through the results and map them to the struct
        while let Some(result) = cursor.try_next().await? {
            if let Ok(view) =
                bson::from_bson::<ClanMateCollectionLogTotalsView>(bson::Bson::Document(result))
            {
                results.push(view);
            }
        }

        return Ok(results);
    }
}
