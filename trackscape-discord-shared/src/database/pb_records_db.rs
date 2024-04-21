use bson::DateTime;
use mockall::predicate::*;
use mongodb::bson::doc;
use mongodb::{bson, Database};
use serde::{Deserialize, Serialize};

use super::PersonalBestRecordsDb;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersonalBestRecordsModel {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub clan_mate_id: bson::oid::ObjectId,
    pub activity_id: bson::oid::ObjectId,
    pub time_in_seconds: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
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
        time_in_seconds: i64,
    ) -> Result<(), mongodb::error::Error> {
        let collection = self
            .db
            .collection::<PersonalBestRecordsModel>(PersonalBestRecordsModel::COLLECTION_NAME);

        let filter = doc! {
            "clan_mate_id": clan_mate_id.clone(),
            "activity_id": activity_id
        };
        match collection.find_one(filter.clone(), None).await? {
            Some(recorded_record) => {
                if time_in_seconds > recorded_record.time_in_seconds {
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
                    time_in_seconds: time_in_seconds.clone(),
                    created_at: bson::DateTime::now(),
                    updated_at: bson::DateTime::now(),
                };
                collection.insert_one(new_pb, None).await?;
                Ok(())
            }
        }
    }
}
