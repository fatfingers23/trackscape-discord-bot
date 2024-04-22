use super::clan_mates::ClanMateModel;
use super::PersonalBestRecordsDb;
use bson::DateTime;
use futures::TryStreamExt;
use mockall::predicate::*;
use mongodb::bson::doc;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestRecordsModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub clan_mate_id: bson::oid::ObjectId,
    pub activity_id: bson::oid::ObjectId,
    pub guild_id: u64,
    pub time_in_seconds: f64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub clan_mate: Option<ClanMateModel>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestRecordsViewModel {
    pub player_name: String,
    pub time_in_seconds: f64,
}

impl PersonalBestRecordsModel {
    pub const COLLECTION_NAME: &'static str = "personal_best_records";
}

impl PersonalBestRecordsDb {
    pub fn new_instance(mongodb: Database) -> Self {
        Self { db: mongodb }
    }

    pub async fn create_or_update_pb_record(
        &self,
        clan_mate_id: bson::oid::ObjectId,
        activity_id: bson::oid::ObjectId,
        guild_id: u64,
        time_in_seconds: f64,
    ) -> Result<(), anyhow::Error> {
        let collection = self
            .db
            .collection::<PersonalBestRecordsModel>(PersonalBestRecordsModel::COLLECTION_NAME);

        let filter = doc! {
            "clan_mate_id": clan_mate_id.clone(),
            "activity_id": activity_id
        };
        match collection.find_one(filter.clone(), None).await? {
            Some(recorded_record) => {
                if time_in_seconds < recorded_record.time_in_seconds {
                    let update = doc! {
                        "$set": {
                            "time_in_seconds": time_in_seconds,
                            "updated_at": bson::DateTime::now()
                        }
                    };
                    collection.update_one(filter, update, None).await?;
                }
                Ok(())
            }
            None => {
                let new_pb = PersonalBestRecordsModel {
                    id: bson::oid::ObjectId::new(),
                    clan_mate_id: clan_mate_id.clone(),
                    activity_id: activity_id.clone(),
                    guild_id,
                    time_in_seconds: time_in_seconds.clone(),
                    created_at: bson::DateTime::now(),
                    updated_at: bson::DateTime::now(),
                    clan_mate: None,
                };
                collection.insert_one(new_pb, None).await?;
                Ok(())
            }
        }
    }

    pub async fn get_pb_records_leaderboard(
        &self,
        activity_id: bson::oid::ObjectId,
        guild_id: u64,
    ) -> Result<Vec<PersonalBestRecordsModel>, anyhow::Error> {
        let collection = self
            .db
            .collection::<PersonalBestRecordsModel>(PersonalBestRecordsModel::COLLECTION_NAME);

        let filter = doc! {
            "guild_id": bson::to_bson(&guild_id).unwrap(),
            "activity_id": activity_id
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
                            "localField": "clan_mate_id",
                            "foreignField": "_id",
                            "as": "clan_mate"
                        }
                    },
                    doc! {
                        "$unwind": "$clan_mate"
                    },
                    doc! {
                        "$sort": {
                            "time_in_seconds": 1,
                            "updated_at": 1
                        }
                    },
                ],
                None,
            )
            .await?;

        let mut results: Vec<PersonalBestRecordsModel> = Vec::new();
        // Iterate through the results and map them to the struct

        while let Some(result) = cursor.try_next().await? {
            println!("{:?}", result);

            if let Ok(view) =
                bson::from_bson::<PersonalBestRecordsModel>(bson::Bson::Document(result))
            {
                results.push(view);
            }
        }
        Ok(results)
    }
}
